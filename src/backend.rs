mod codegen;
mod fixup;
mod replace_pseudos;

use crate::models::assembly as asm;
use crate::models::tacky;

pub fn codegen(program: tacky::Program) -> asm::Program<asm::Plain> {
    let program = codegen::gen_program(program);
    let program = replace_pseudos::replace_pseudos(program);
    fixup::fixup(program)
}
