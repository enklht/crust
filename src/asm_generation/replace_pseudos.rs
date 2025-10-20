use crate::models::assembly::*;
use crate::models::symbol::{Symbol, Symbols};
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

    fn get(&mut self, ident: String) -> usize {
        *self.offset_map.entry(ident).or_insert_with(|| {
            self.current_offset += 4;
            self.current_offset
        })
    }
}

pub fn replace_pseudos(program: Program<Plain>, symbols: &Symbols) -> Program<LastOffset> {
    match program {
        Program::Program(top_levels) => Program::Program(
            top_levels
                .into_iter()
                .map(|top_level| replace_pseudos_in_top_level(top_level, symbols))
                .collect(),
        ),
    }
}

fn replace_pseudos_in_top_level(
    top_level: TopLevel<Plain>,
    symbols: &Symbols,
) -> TopLevel<LastOffset> {
    let state = &mut State::new();
    match top_level {
        TopLevel::Function(name, global, instructions, ()) => TopLevel::Function(
            name,
            global,
            instructions
                .into_iter()
                .map(|i| replace_pseudos_in_instruction(i, state, symbols))
                .collect(),
            state.current_offset,
        ),
        TopLevel::StaticVar(name, global, init) => TopLevel::StaticVar(name, global, init),
    }
}

fn replace_pseudos_in_instruction(
    instruction: Instruction,
    state: &mut State,
    symbols: &Symbols,
) -> Instruction {
    match instruction {
        Instruction::Mov(src, dst) => {
            let src = replace_operand(src, state, symbols);
            let dst = replace_operand(dst, state, symbols);
            Instruction::Mov(src, dst)
        }
        Instruction::Unary(op, dst) => {
            let dst = replace_operand(dst, state, symbols);
            Instruction::Unary(op, dst)
        }
        Instruction::Binary(op, src, dst) => {
            let src = replace_operand(src, state, symbols);
            let dst = replace_operand(dst, state, symbols);
            Instruction::Binary(op, src, dst)
        }
        Instruction::Idiv(op) => {
            let op = replace_operand(op, state, symbols);
            Instruction::Idiv(op)
        }
        Instruction::Shl(dst) => {
            let dst = replace_operand(dst, state, symbols);
            Instruction::Shl(dst)
        }
        Instruction::Shr(dst) => {
            let dst = replace_operand(dst, state, symbols);
            Instruction::Shr(dst)
        }
        Instruction::Cmp(src, dst) => {
            let src = replace_operand(src, state, symbols);
            let dst = replace_operand(dst, state, symbols);
            Instruction::Cmp(src, dst)
        }
        Instruction::SetCC(val, target) => {
            let target = replace_operand(target, state, symbols);
            Instruction::SetCC(val, target)
        }
        Instruction::Push(op) => {
            let op = replace_operand(op, state, symbols);
            Instruction::Push(op)
        }
        other => other,
    }
}

fn replace_operand(operand: Operand, state: &mut State, symbols: &Symbols) -> Operand {
    match operand {
        Operand::Pseudo(ident) => match symbols.get(&ident) {
            Some(Symbol::StaticVar { .. }) => Operand::Data(ident),
            _ => Operand::Stack(-(state.get(ident) as isize)),
        },
        other => other,
    }
}
