mod fixup;
mod replace_pseudos;
mod tacky_to_asm;

use crate::models::assembly as asm;
use crate::models::symbol::Symbols;
use crate::models::tacky;

pub fn codegen(program: tacky::Program, symbols: &Symbols) -> asm::Program<asm::Plain> {
    let program = tacky_to_asm::gen_program(program);
    let program = replace_pseudos::replace_pseudos(program, symbols);
    fixup::fixup(program)
}
