use super::asm;

#[derive(Default)]
struct State {
    current_offset: isize,
    offset_map: std::collections::HashMap<String, isize>,
}

impl State {
    fn get(&mut self, ident: String) -> isize {
        *self.offset_map.entry(ident).or_insert_with(|| {
            self.current_offset -= 4;
            self.current_offset
        })
    }
}

pub fn replace_pseudos(program: asm::Program) -> (asm::Program, isize) {
    match program {
        asm::Program::Program(function) => {
            let (function, offset) = replace_pseudos_in_function(function);
            (asm::Program::Program(function), offset)
        }
    }
}

fn replace_pseudos_in_function(function: asm::Function) -> (asm::Function, isize) {
    let state = &mut State::default();
    let function = match function {
        asm::Function::Function(name, instructions) => asm::Function::Function(
            name,
            instructions
                .into_iter()
                .map(|i| replace_pseudos_in_instruction(i, state))
                .collect(),
        ),
    };
    (function, state.current_offset)
}

fn replace_pseudos_in_instruction(
    instruction: asm::Instruction,
    state: &mut State,
) -> asm::Instruction {
    match instruction {
        asm::Instruction::Mov(src, dst) => {
            let src = replace_operand(src, state);
            let dst = replace_operand(dst, state);
            asm::Instruction::Mov(src, dst)
        }
        asm::Instruction::Unary(op, val) => {
            let val = replace_operand(val, state);
            asm::Instruction::Unary(op, val)
        }
        asm::Instruction::Ret => asm::Instruction::Ret,
        asm::Instruction::AllocateStack(_) => unreachable!(),
    }
}

fn replace_operand(operand: asm::Operand, state: &mut State) -> asm::Operand {
    match operand {
        asm::Operand::Pseudo(ident) => asm::Operand::Stack(state.get(ident)),
        other => other,
    }
}
