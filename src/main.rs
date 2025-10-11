mod codeemit;
mod codegen;
mod lex;
mod models;
mod parse;

use bpaf::*;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    #[bpaf(external(compiler_mode), fallback(CompilerMode::Full))]
    mode: CompilerMode,
    #[bpaf(short)]
    output: PathBuf,
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

    let source = fs::read_to_string(&opts.input)?;

    let tokens = lex::lex(&source)?;

    if opts.mode == CompilerMode::Lex {
        return Ok(());
    }

    let program = parse::parse(tokens)?;

    if opts.mode == CompilerMode::Parse {
        return Ok(());
    }

    let assembly_ast = codegen::codegen(program);

    if opts.mode == CompilerMode::Codegen {
        return Ok(());
    }

    let assembly = codeemit::emit(assembly_ast);
    fs::write(&opts.output, assembly)?;
    Ok(())
}
