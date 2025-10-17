use super::asm::*;
use rustc_hash::FxHashMap;

#[derive(Default)]
struct State {
    current_offset: isize,
    offset_map: FxHashMap<String, isize>,
}

impl State {
    fn get(&mut self, ident: String) -> isize {
        *self.offset_map.entry(ident).or_insert_with(|| {
            self.current_offset -= 4;
            self.current_offset
        })
    }
}

pub fn replace_pseudos(program: Program) -> (Program, isize) {
    match program {
        Program::Program(function) => {
            let (function, offset) = replace_pseudos_in_function(function);
            (Program::Program(function), offset)
        }
    }
}

fn replace_pseudos_in_function(function: Function) -> (Function, isize) {
    let state = &mut State::default();
    let function = match function {
        Function::Function(name, instructions) => Function::Function(
            name,
            instructions
                .into_iter()
                .map(|i| replace_pseudos_in_instruction(i, state))
                .collect(),
        ),
    };
    (function, state.current_offset)
}

fn replace_pseudos_in_instruction(instruction: Instruction, state: &mut State) -> Instruction {
    match instruction {
        Instruction::Mov(src, dst) => {
            let src = replace_operand(src, state);
            let dst = replace_operand(dst, state);
            Instruction::Mov(src, dst)
        }
        Instruction::Unary(op, dst) => {
            let dst = replace_operand(dst, state);
            Instruction::Unary(op, dst)
        }
        Instruction::Binary(op, src, dst) => {
            let src = replace_operand(src, state);
            let dst = replace_operand(dst, state);
            Instruction::Binary(op, src, dst)
        }
        Instruction::Idiv(op) => {
            let op = replace_operand(op, state);
            Instruction::Idiv(op)
        }
        Instruction::Shl(dst) => {
            let dst = replace_operand(dst, state);
            Instruction::Shl(dst)
        }
        Instruction::Shr(dst) => {
            let dst = replace_operand(dst, state);
            Instruction::Shr(dst)
        }
        Instruction::Cmp(src, dst) => {
            let src = replace_operand(src, state);
            let dst = replace_operand(dst, state);
            Instruction::Cmp(src, dst)
        }
        Instruction::SetCC(val, target) => {
            let target = replace_operand(target, state);
            Instruction::SetCC(val, target)
        }
        Instruction::AllocateStack(_) => unreachable!(),
        other => other,
    }
}

fn replace_operand(operand: Operand, state: &mut State) -> Operand {
    match operand {
        Operand::Pseudo(ident) => Operand::Stack(state.get(ident)),
        other => other,
    }
}
