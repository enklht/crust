mod fixup;
mod replace_pseudos;
mod tacky_to_asm;

use crate::models::asm_symbol::AsmSymbols;
use crate::models::assembly as asm;
use crate::models::symbol::Symbols;
use crate::models::tacky;
use crate::models::typ::Types;
use fixup::fixup;
use replace_pseudos::replace_pseudos;
use tacky_to_asm::gen_program;

macro_rules! reg {
    ($reg: ident) => {
        Operand::Reg(Reg::$reg)
    };
}

macro_rules! imm {
    ($imm: expr) => {
        Operand::Imm($imm)
    };
}

use imm;
use reg;

pub fn codegen(
    program: tacky::Program,
    symbols: Symbols,
    types: &Types,
) -> (asm::Program, AsmSymbols) {
    let (mut program, asm_symbols) = gen_program(program, symbols, types);

    replace_pseudos(&mut program, &asm_symbols);

    fixup(&mut program);

    (program, asm_symbols)
}
