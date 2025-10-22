use crate::models::asm_symbol::AsmSymbol;
use crate::models::asm_symbol::AsmSymbols;
use crate::models::assembly::*;
use rustc_hash::FxHashMap;

struct State {
    current_offset: usize,
    offset_map: FxHashMap<String, usize>,
}

impl State {
    fn new() -> State {
        State {
            current_offset: 0,
            offset_map: FxHashMap::default(),
        }
    }

    fn longword(&mut self, ident: &str) -> usize {
        *self.offset_map.entry(ident.to_string()).or_insert_with(|| {
            self.current_offset += 4;
            self.current_offset
        })
    }

    fn quadword(&mut self, ident: &str) -> usize {
        *self.offset_map.entry(ident.to_string()).or_insert_with(|| {
            self.current_offset += 8;
            self.current_offset = self.current_offset.next_multiple_of(8);
            self.current_offset
        })
    }
}

pub fn replace_pseudos(program: &mut Program, symbols: &AsmSymbols) {
    match program {
        Program::Program(top_levels) => top_levels
            .iter_mut()
            .for_each(|top_level| replace_pseudos_in_top_level(top_level, symbols)),
    }
}

fn replace_pseudos_in_top_level(top_level: &mut TopLevel, symbols: &AsmSymbols) {
    let state = &mut State::new();

    if let TopLevel::Function(_, _, instructions, last_offset) = top_level {
        instructions
            .iter_mut()
            .for_each(|i| replace_pseudos_in_instruction(i, state, symbols));
        *last_offset = Some(state.current_offset);
    }
}

fn replace_pseudos_in_instruction(
    instruction: &mut Instruction,
    state: &mut State,
    symbols: &AsmSymbols,
) {
    match instruction {
        Instruction::Mov(src, dst, _) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Movsx(src, dst) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Unary(_, dst, _) => {
            replace_operand(dst, state, symbols);
        }
        Instruction::Binary(_, src, dst, _) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Idiv(op, _) => {
            replace_operand(op, state, symbols);
        }
        Instruction::Cmp(src, dst, _) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::SetCC(_, target) => {
            replace_operand(target, state, symbols);
        }
        Instruction::Push(op) => {
            replace_operand(op, state, symbols);
        }
        _ => {}
    }
}

fn replace_operand(operand: &mut Operand, state: &mut State, symbols: &AsmSymbols) {
    if let Operand::Pseudo(ident) = operand {
        match symbols.get(ident).unwrap() {
            AsmSymbol::ObjEntry(_, true) => *operand = Operand::Data(ident.clone()),
            AsmSymbol::ObjEntry(AssemblyType::Longword, false) => {
                *operand = Operand::Stack(-(state.longword(ident) as isize))
            }
            AsmSymbol::ObjEntry(AssemblyType::Quadword, false) => {
                *operand = Operand::Stack(-(state.quadword(ident) as isize))
            }
            _ => unreachable!(),
        }
    }
}
