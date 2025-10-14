mod backend;
mod emit;
mod lex;
mod models;
mod parse;
mod semantic_analysis;
mod tacky_gen;
mod utils;

use bpaf::*;
use std::fs;
use std::path::PathBuf;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Options = options().run();

    let source = fs::read_to_string(&opts.input)?;

    let tokens = lex::lex(&source)?;

    if opts.mode == CompilerMode::Lex {
        println!("{:#?}", tokens);
        return Ok(());
    }

    let ast = parse::parse(tokens)?;

    if opts.mode == CompilerMode::Parse {
        println!("{:#?}", ast);
        return Ok(());
    }

    let validated_ast = semantic_analysis::validate(ast)?;

    if opts.mode == CompilerMode::Validate {
        println!("{:#?}", validated_ast);
        return Ok(());
    }

    let tacky = tacky_gen::convert_program(validated_ast);

    if opts.mode == CompilerMode::Tacky {
        println!("{:#?}", tacky);
        return Ok(());
    }

    let asm_ast = backend::codegen(tacky);

    if opts.mode == CompilerMode::Codegen {
        println!("{:#?}", asm_ast);
        return Ok(());
    }

    let asm_text = emit::emit(asm_ast);
    let output = opts.output.unwrap_or(opts.input.with_extension("s"));
    fs::write(&output, asm_text)?;

    Ok(())
}
