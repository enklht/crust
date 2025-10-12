mod backend;
mod codeemit;
mod lex;
mod models;
mod parse;
mod tacky_gen;

use bpaf::*;
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

    let tokens = lex::lex(&source)?;

    if opts.mode == CompilerMode::Lex {
        return Ok(());
    }

    let program = parse::parse(tokens)?;

    if opts.mode == CompilerMode::Parse {
        return Ok(());
    }

    let tacky = tacky_gen::convert_program(program);

    if opts.mode == CompilerMode::Tacky {
        return Ok(());
    }

    let assembly_ast = backend::codegen(tacky);

    if opts.mode == CompilerMode::Codegen {
        return Ok(());
    }

    // let assembly = codeemit::emit(assembly_ast);
    // let output = opts.output.unwrap_or(opts.input.with_extension("s"));
    // fs::write(&output, assembly)?;

    Ok(())
}
