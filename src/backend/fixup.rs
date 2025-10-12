use super::asm::*;

pub fn fixup(program: Program, last_offset: isize) -> Program {
    match program {
        Program::Program(function) => Program::Program(fixup_function(function, last_offset)),
    }
}

fn fixup_function(function: Function, last_offset: isize) -> Function {
    match function {
        Function::Function(name, instructions) => {
            let mut fixed = vec![Instruction::AllocateStack(-last_offset as usize)];

            for instruction in instructions {
                emit_instruction(instruction, &mut fixed);
            }

            Function::Function(name, fixed)
        }
    }
}

fn emit_instruction(instruction: Instruction, acc: &mut Vec<Instruction>) {
    match instruction {
        Instruction::Mov(src @ Operand::Stack(_), dst @ Operand::Stack(_)) => {
            acc.extend([
                Instruction::Mov(src, Operand::Reg(Reg::R10)),
                Instruction::Mov(Operand::Reg(Reg::R10), dst),
            ]);
        }
        Instruction::Binary(
            op @ (BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor),
            src @ Operand::Stack(_),
            dst @ Operand::Stack(_),
        ) => {
            acc.extend([
                Instruction::Mov(src, Operand::Reg(Reg::R10)),
                Instruction::Binary(op, Operand::Reg(Reg::R10), dst),
            ]);
        }
        Instruction::Binary(BinaryOp::Multiply, src, dst @ Operand::Stack(_)) => {
            acc.extend([
                Instruction::Mov(dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Binary(BinaryOp::Multiply, src, Operand::Reg(Reg::R11)),
                Instruction::Mov(Operand::Reg(Reg::R11), dst),
            ]);
        }
        Instruction::Idiv(op @ Operand::Imm(_)) => {
            acc.extend([
                Instruction::Mov(op, Operand::Reg(Reg::R10)),
                Instruction::Idiv(Operand::Reg(Reg::R10)),
            ]);
        }
        other => {
            acc.push(other);
        }
    }
}
