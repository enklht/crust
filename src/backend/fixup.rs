use super::asm;

pub fn fixup(program: asm::Program, last_offset: isize) -> asm::Program {
    match program {
        asm::Program::Program(function) => {
            asm::Program::Program(fixup_function(function, last_offset))
        }
    }
}

fn fixup_function(function: asm::Function, last_offset: isize) -> asm::Function {
    match function {
        asm::Function::Function(name, instructions) => {
            let mut fixed = vec![asm::Instruction::AllocateStack(-last_offset as usize)];

            for instruction in instructions {
                emit_instruction(instruction, &mut fixed);
            }

            asm::Function::Function(name, fixed)
        }
    }
}

fn emit_instruction(instruction: asm::Instruction, acc: &mut Vec<asm::Instruction>) {
    match instruction {
        asm::Instruction::Mov(src @ asm::Operand::Stack(_), dst @ asm::Operand::Stack(_)) => {
            acc.push(asm::Instruction::Mov(src, asm::Operand::Reg(asm::Reg::R10)));
            acc.push(asm::Instruction::Mov(asm::Operand::Reg(asm::Reg::R10), dst));
        }
        other => {
            acc.push(other);
        }
    }
}
