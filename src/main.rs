mod backend;
mod emit;
mod lex;
mod models;
mod parse;
mod tacky_gen;

use crate::{lex::LexError, models::token::Token};
use bpaf::*;
use logos::Logos;
use std::{fs, path::PathBuf};

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
    Tacky,
    Codegen,
    Full,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Options = options().run();

    let source = fs::read_to_string(&opts.input)?;

    let lexer = Token::lexer(&source).peekable();

    if opts.mode == CompilerMode::Lex {
        println!("{:?}", lexer);
        return Ok(());
    }

    let ast = parse::parse(lexer)?;

    if opts.mode == CompilerMode::Parse {
        println!("{:?}", ast);
        return Ok(());
    }

    let tacky = tacky_gen::convert_program(ast);

    if opts.mode == CompilerMode::Tacky {
        println!("{:?}", tacky);
        return Ok(());
    }

    let asm_ast = backend::codegen(tacky);

    if opts.mode == CompilerMode::Codegen {
        println!("{:?}", asm_ast);
        return Ok(());
    }

    let asm_text = emit::emit(asm_ast);
    let output = opts.output.unwrap_or(opts.input.with_extension("s"));
    fs::write(&output, asm_text)?;

    Ok(())
}
