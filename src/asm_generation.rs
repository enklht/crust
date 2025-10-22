mod fixup;
mod replace_pseudos;
mod tacky_to_asm;

use crate::models::asm_symbol::{AsmSymbols, from_symbols};
use crate::models::assembly::{self as asm};
use crate::models::symbol::Symbols;
use crate::models::tacky;
use fixup::fixup;
use replace_pseudos::replace_pseudos;
use tacky_to_asm::gen_program;

macro_rules! reg {
    ($reg: ident) => {
        Operand::Reg(Reg::$reg)
    };
}

use reg;

pub fn codegen(program: tacky::Program, symbols: Symbols) -> (asm::Program, AsmSymbols) {
    let mut program = gen_program(program, &symbols);

    let asm_symbols = from_symbols(symbols);

    replace_pseudos(&mut program, &asm_symbols);
    fixup(&mut program);

    (program, asm_symbols)
}
