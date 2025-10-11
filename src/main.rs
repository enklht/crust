mod codegen;
mod lex;
mod parser;
mod token;

use bpaf::*;
use lex::Lexer;
use std::{fs, path::PathBuf};

use crate::parser::parse;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    #[bpaf(external(compiler_mode), fallback(CompilerMode::Full))]
    mode: CompilerMode,
    #[bpaf(positional("file"))]
    input: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Bpaf)]
enum CompilerMode {
    Lex,
    Parse,
    Codegen,
    Full,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Options = options().run();

    // Read the input file
    let source = fs::read_to_string(&opts.input)?;

    let lexer = Lexer::new();
    let tokens = lexer.lex(&source)?;

    if opts.mode == CompilerMode::Lex {
        return Ok(());
    }

    let ast = parse(tokens)?;

    if opts.mode == CompilerMode::Parse {
        return Ok(());
    }

    todo!()
}
