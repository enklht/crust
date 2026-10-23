mod asm_generation;
mod code_emission;
mod error_reporting;
mod lexer;
mod models;
mod parser;
mod semantic_analysis;
mod tacky_generation;
mod utils;

use bpaf::*;
use chumsky::Parser;
use chumsky::input::Input;
use error_reporting::report_errors;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    #[bpaf(external(compiler_mode), fallback(CompilerMode::Full))]
    mode: CompilerMode,
    #[bpaf(short)]
    output: Option<PathBuf>,
    #[bpaf(positional("file"))]
    input: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Bpaf)]
enum CompilerMode {
    Lex,
    Parse,
    Validate,
    Tacky,
    Codegen,
    Emit,
    Full,
}

#[derive(Error, Debug)]
enum CrustError {
    #[error("cannot read '{path}': {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write '{path}': {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Validation {
        path: PathBuf,
        source: semantic_analysis::ValidationError,
    },
}

fn main() -> ExitCode {
    match compile(options().run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn compile(opts: Options) -> Result<(), CrustError> {
    let fname = &opts.input.display().to_string();
    let source = &fs::read_to_string(&opts.input).map_err(|source| CrustError::Read {
        path: opts.input.clone(),
        source,
    })?;

    let lexer = lexer::lexer();

    let tokens = match lexer.parse(source).into_result() {
        Ok(tokens) => tokens,
        Err(errs) => {
            report_errors(fname, &errs, source);
        }
    };

    if opts.mode == CompilerMode::Lex {
        tokens.iter().for_each(|token| println!("{token:?},"));
        return Ok(());
    }

    let mut state = chumsky::inspector::SimpleState(models::typ::Types::new());
    let ast = match parser::parser()
        .parse_with_state(
            tokens
                .as_slice()
                .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
            &mut state,
        )
        .into_result()
    {
        Ok(ast) => ast,
        Err(errs) => {
            report_errors(fname, &errs, source);
        }
    };

    let mut types = state.0;

    if opts.mode == CompilerMode::Parse {
        println!("{ast:#?}");
        return Ok(());
    }

    let (validated_ast, mut symbols) =
        semantic_analysis::validate(ast, &mut types).map_err(|source| CrustError::Validation {
            path: opts.input.clone(),
            source,
        })?;

    if opts.mode == CompilerMode::Validate {
        println!("{validated_ast:#?}");
        return Ok(());
    }

    let tacky = tacky_generation::convert_program(validated_ast, &mut symbols, &mut types);

    if opts.mode == CompilerMode::Tacky {
        println!("{tacky:#?}");
        return Ok(());
    }

    let (asm_ast, asm_symbols) = asm_generation::codegen(tacky, symbols, &types);

    if opts.mode == CompilerMode::Codegen {
        println!("{asm_ast:#?}");
        return Ok(());
    }

    let asm_text = code_emission::emit(asm_ast, &asm_symbols);

    if opts.mode == CompilerMode::Emit {
        asm_text
            .lines()
            .enumerate()
            .for_each(|(i, line)| println!("{}\t{}", i + 1, line));
        return Ok(());
    }

    let output = opts.output.unwrap_or(opts.input.with_extension("s"));
    fs::write(&output, asm_text).map_err(|source| CrustError::Write {
        path: output,
        source,
    })?;

    Ok(())
}
