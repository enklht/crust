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

    fn get_offset_or_insert(&mut self, ident: &str, asm_type: &AsmType) -> usize {
        *self.offset_map.entry(ident.to_string()).or_insert_with(|| {
            let size = asm_type.size();
            let alignment = asm_type.alignment();
            self.current_offset = (self.current_offset + size).next_multiple_of(alignment);
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
        Instruction::MovZeroExtend(src, dst) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Lea(src, dst) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Cvttsd2si(src, dst, _) => {
            replace_operand(src, state, symbols);
            replace_operand(dst, state, symbols);
        }
        Instruction::Cvtsi2sd(src, dst, _) => {
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
        Instruction::Div(op, _) => {
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
    match operand {
        Operand::Pseudo(ident) => match symbols.get(ident).unwrap() {
            AsmSymbol::ObjEntry(_, true, _) => {
                *operand = Operand::Data(ident);
            }
            AsmSymbol::ObjEntry(asm_type, false, _) => {
                *operand = Operand::Memory(
                    Reg::BP,
                    -(state.get_offset_or_insert(ident, asm_type) as isize),
                )
            }
            _ => unreachable!(),
        },
        Operand::PseudoMem(ident, offset) => match symbols.get(ident).unwrap() {
            AsmSymbol::ObjEntry(_, true, _) => {
                if *offset == 0 {
                    *operand = Operand::Data(ident);
                } else {
                    unreachable!();
                }
            }
            AsmSymbol::ObjEntry(asm_type, false, _) => {
                *operand = Operand::Memory(
                    Reg::BP,
                    -((state.get_offset_or_insert(ident, asm_type) - *offset) as isize),
                )
            }
            _ => unreachable!(),
        },
        _ => {}
    }
}
