use crate::models::assembly::*;

pub fn fixup(program: Program<LastOffset>) -> Program<Plain> {
    match program {
        Program::Program(func_defs) => {
            Program::Program(func_defs.into_iter().map(fixup_function).collect())
        }
    }
}

fn fixup_function(function: TopLevel<LastOffset>) -> TopLevel<Plain> {
    match function {
        TopLevel::Function(name, global, instructions, last_offset) => {
            let mut fixed = vec![Instruction::AllocateStack(last_offset.next_multiple_of(16))];

            for instruction in instructions {
                emit_instruction(instruction, &mut fixed);
            }

            TopLevel::Function(name, global, fixed, ())
        }
        TopLevel::StaticVar(name, global, init) => TopLevel::StaticVar(name, global, init),
    }
}

fn emit_instruction(instruction: Instruction, acc: &mut Vec<Instruction>) {
    match instruction {
        Instruction::Mov(
            src @ (Operand::Stack(_) | Operand::Data(_)),
            dst @ (Operand::Stack(_) | Operand::Data(_)),
        ) => {
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
            src @ (Operand::Stack(_) | Operand::Data(_)),
            dst @ (Operand::Stack(_) | Operand::Data(_)),
        ) => {
            acc.extend([
                Instruction::Mov(src, Operand::Reg(Reg::R10)),
                Instruction::Binary(op, Operand::Reg(Reg::R10), dst),
            ]);
        }
        Instruction::Binary(
            BinaryOp::Multiply,
            src,
            dst @ (Operand::Stack(_) | Operand::Data(_)),
        ) => {
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
        Instruction::Cmp(
            src @ (Operand::Stack(_) | Operand::Data(_)),
            dst @ (Operand::Stack(_) | Operand::Data(_)),
        ) => {
            acc.extend([
                Instruction::Mov(src, Operand::Reg(Reg::R10)),
                Instruction::Cmp(Operand::Reg(Reg::R10), dst),
            ]);
        }
        Instruction::Cmp(src, dst @ Operand::Imm(_)) => {
            acc.extend([
                Instruction::Mov(dst, Operand::Reg(Reg::R11)),
                Instruction::Cmp(src, Operand::Reg(Reg::R11)),
            ]);
        }
        Instruction::Push(op @ (Operand::Stack(_) | Operand::Data(_))) => acc.extend([
            Instruction::Mov(op, Operand::Reg(Reg::AX)),
            Instruction::Push(Operand::Reg(Reg::AX)),
        ]),
        other => {
            acc.push(other);
        }
    }
}
