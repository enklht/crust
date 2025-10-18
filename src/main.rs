mod backend;
mod emit;
mod error_report;
mod lex;
mod models;
mod parse;
mod semantic_analysis;
// mod tacky_gen;
mod utils;

use bpaf::*;
use chumsky::Parser;
use chumsky::input::Input;
use std::fs;
use std::path::PathBuf;
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
    Full,
}

#[derive(Error, Debug)]
enum CrustError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ValidationError(#[from] crate::semantic_analysis::ValidationError),
}

fn main() -> Result<(), CrustError> {
    let opts: Options = options().run();

    println!("{opts:?}");

    let fname = &opts.input.display().to_string();
    let source = &fs::read_to_string(&opts.input)?;

    let lexer = lex::lexer();

    let tokens = match lexer.parse(source).into_result() {
        Ok(tokens) => tokens,
        Err(errs) => {
            error_report::report_errors(fname, &errs, source);
        }
    };

    if opts.mode == CompilerMode::Lex {
        tokens.iter().for_each(|token| println!("{token:?},"));
        return Ok(());
    }

    let ast = match parse::parser()
        .parse(
            tokens
                .as_slice()
                .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
        )
        .into_result()
    {
        Ok(ast) => ast,
        Err(errs) => {
            error_report::report_errors(fname, &errs, source);
        }
    };

    if opts.mode == CompilerMode::Parse {
        println!("{:#?}", ast);
        return Ok(());
    }

    let validated_ast = semantic_analysis::validate(ast)?;

    if opts.mode == CompilerMode::Validate {
        println!("{:#?}", validated_ast);
        return Ok(());
    }

    // let tacky = tacky_gen::convert_program(validated_ast);
    //
    // if opts.mode == CompilerMode::Tacky {
    //     println!("{:?}", tacky);
    //     return Ok(());
    // }
    //
    // let asm_ast = backend::codegen(tacky);
    //
    // if opts.mode == CompilerMode::Codegen {
    //     println!("{:#?}", asm_ast);
    //     return Ok(());
    // }
    //
    // let asm_text = emit::emit(asm_ast);
    //
    // let output = opts.output.unwrap_or(opts.input.with_extension("s"));
    // fs::write(&output, asm_text)?;

    Ok(())
}
