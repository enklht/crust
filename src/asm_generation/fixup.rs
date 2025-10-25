use super::reg;
use crate::models::assembly::*;

macro_rules! memory {
    () => {
        Operand::Stack(_) | Operand::Data(_)
    };
}

fn fits_in_i32(i: u64) -> bool {
    (i32::MIN as i64..i32::MAX as i64).contains(&(i as i64))
}

pub fn fixup(program: &mut Program) {
    match program {
        Program::Program(top_level) => top_level.iter_mut().for_each(fixup_top_level),
    }
}

fn fixup_top_level(top_level: &mut TopLevel) {
    if let TopLevel::Function(_, _, instructions, last_offset) = top_level {
        let old_instructions = std::mem::take(instructions);

        instructions.extend([Instruction::Binary(
            BinaryOp::Subtract,
            Operand::Imm(last_offset.unwrap().next_multiple_of(16) as u64),
            reg!(SP),
            AsmType::Quadword,
        )]);

        for instruction in old_instructions {
            emit_instruction(instruction, instructions);
        }
    }
}

fn emit_instruction(instruction: Instruction, acc: &mut Vec<Instruction>) {
    use AsmType::*;
    match instruction {
        Instruction::Mov(src @ memory!(), dst @ memory!(), asm_type) => {
            let tmp = match asm_type {
                Double => reg!(XMM14),
                _ => reg!(R10),
            };

            acc.extend([
                Instruction::Mov(src, tmp.clone(), asm_type),
                Instruction::Mov(tmp, dst, asm_type),
            ]);
        }
        Instruction::Mov(src @ Operand::Imm(_), dst, Quadword) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), Quadword),
                Instruction::Mov(reg!(R10), dst, Quadword),
            ]);
        }
        // TODO: is this fits_in_i32 necessary?
        Instruction::Mov(Operand::Imm(i), dst, Longword) if !fits_in_i32(i) => {
            acc.extend([
                Instruction::Mov(Operand::Imm(i & 0xffffffff), reg!(R10), Longword),
                Instruction::Mov(reg!(R10), dst, Longword),
            ]);
        }
        Instruction::Movsx(src @ Operand::Imm(_), dst @ memory!()) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), Longword),
                Instruction::Movsx(reg!(R10), reg!(R11)),
                Instruction::Mov(reg!(R11), dst, Quadword),
            ]);
        }
        Instruction::Movsx(src @ Operand::Imm(_), dst) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), Longword),
                Instruction::Movsx(reg!(R10), dst),
            ]);
        }
        Instruction::Movsx(src, dst @ memory!()) => {
            acc.extend([
                Instruction::Movsx(src, reg!(R11)),
                Instruction::Mov(reg!(R11), dst, Quadword),
            ]);
        }
        Instruction::MovZeroExtend(src, dst @ Operand::Reg(_)) => {
            acc.push(Instruction::Mov(src, dst, Longword))
        }
        Instruction::MovZeroExtend(src, dst @ memory!()) => {
            acc.extend([
                Instruction::Mov(src, reg!(R11), Longword),
                Instruction::Mov(reg!(R11), dst, Quadword),
            ]);
        }
        // Instruction::Cvttsd2si(src @ Operand::Imm(_), dst @ memory!(), asm_type) => {
        //     acc.extend([
        //         Instruction::Mov(src, reg!(XMM14), Double),
        //         Instruction::Cvttsd2si(reg!(XMM14), reg!(R11), asm_type),
        //         Instruction::Mov(reg!(R11), dst, asm_type),
        //     ]);
        // }
        // Instruction::Cvttsd2si(src @ Operand::Imm(_), dst, asm_type) => {
        //     acc.extend([
        //         Instruction::Mov(src, reg!(XMM14), Double),
        //         Instruction::Cvttsd2si(reg!(XMM14), dst, asm_type),
        //     ]);
        // }
        Instruction::Cvttsd2si(src, dst @ memory!(), asm_type) => {
            acc.extend([
                Instruction::Cvttsd2si(src, reg!(R11), asm_type),
                Instruction::Mov(reg!(R11), dst, asm_type),
            ]);
        }
        Instruction::Cvtsi2sd(src @ Operand::Imm(_), dst @ memory!(), asm_type) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Cvtsi2sd(reg!(R10), reg!(XMM15), asm_type),
                Instruction::Mov(reg!(XMM15), dst, Double),
            ]);
        }
        Instruction::Cvtsi2sd(src @ Operand::Imm(_), dst, asm_type) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Cvtsi2sd(reg!(R10), dst, asm_type),
            ]);
        }
        Instruction::Cvtsi2sd(src, dst @ memory!(), asm_type) => {
            acc.extend([
                Instruction::Cvtsi2sd(src, reg!(XMM15), asm_type),
                Instruction::Mov(reg!(XMM15), dst, Double),
            ]);
        }
        Instruction::Binary(
            op @ (BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor),
            src,
            dst @ (memory!() | Operand::Imm(_)),
            Double,
        ) => {
            acc.extend([
                Instruction::Mov(dst.clone(), reg!(XMM15), Double),
                Instruction::Binary(op, src, reg!(XMM15), Double),
                Instruction::Mov(reg!(XMM15), dst, Double),
            ]);
        }
        Instruction::Binary(
            op @ (BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor),
            src @ memory!(),
            dst @ memory!(),
            asm_type,
        ) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Binary(op, reg!(R10), dst, asm_type),
            ]);
        }
        Instruction::Binary(
            op @ (BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor),
            src @ Operand::Imm(i),
            dst,
            asm_type,
        ) if !fits_in_i32(i) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Binary(op, reg!(R10), dst, asm_type),
            ]);
        }
        Instruction::Binary(
            BinaryOp::Multiply,
            src @ Operand::Imm(i),
            dst @ memory!(),
            Quadword,
        ) if !fits_in_i32(i) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), Quadword),
                Instruction::Mov(dst.clone(), reg!(R11), Quadword),
                Instruction::Binary(BinaryOp::Multiply, reg!(R10), reg!(R11), Quadword),
                Instruction::Mov(reg!(R11), dst, Quadword),
            ]);
        }
        Instruction::Binary(BinaryOp::Multiply, src @ Operand::Imm(i), dst, Quadword)
            if !fits_in_i32(i) =>
        {
            acc.extend([
                Instruction::Mov(src, reg!(R10), Quadword),
                Instruction::Binary(BinaryOp::Multiply, reg!(R10), dst, Quadword),
            ]);
        }
        Instruction::Binary(BinaryOp::Multiply, src, dst @ memory!(), asm_type) => {
            acc.extend([
                Instruction::Mov(dst.clone(), reg!(R11), asm_type),
                Instruction::Binary(BinaryOp::Multiply, src, reg!(R11), asm_type),
                Instruction::Mov(reg!(R11), dst, asm_type),
            ]);
        }
        Instruction::Idiv(op @ Operand::Imm(_), asm_type) => {
            acc.extend([
                Instruction::Mov(op, reg!(R10), asm_type),
                Instruction::Idiv(reg!(R10), asm_type),
            ]);
        }
        Instruction::Div(op @ Operand::Imm(_), asm_type) => {
            acc.extend([
                Instruction::Mov(op, reg!(R10), asm_type),
                Instruction::Div(reg!(R10), asm_type),
            ]);
        }
        Instruction::Cmp(src, dst @ (memory!() | Operand::Imm(_)), Double) => {
            acc.extend([
                Instruction::Mov(dst, reg!(XMM15), Double),
                Instruction::Cmp(src, reg!(XMM15), Double),
            ]);
        }
        Instruction::Cmp(src @ memory!(), dst @ memory!(), asm_type) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Cmp(reg!(R10), dst, asm_type),
            ]);
        }
        Instruction::Cmp(src @ Operand::Imm(i), dst @ Operand::Imm(_), asm_type)
            if !fits_in_i32(i) =>
        {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Mov(dst, reg!(R11), asm_type),
                Instruction::Cmp(reg!(R10), reg!(R11), asm_type),
            ]);
        }
        Instruction::Cmp(src @ Operand::Imm(i), dst, asm_type) if !fits_in_i32(i) => {
            acc.extend([
                Instruction::Mov(src, reg!(R10), asm_type),
                Instruction::Cmp(reg!(R10), dst, asm_type),
            ]);
        }
        Instruction::Cmp(src, dst @ Operand::Imm(_), asm_type) => {
            acc.extend([
                Instruction::Mov(dst, reg!(R11), asm_type),
                Instruction::Cmp(src, reg!(R11), asm_type),
            ]);
        }
        Instruction::Push(op @ memory!()) => {
            acc.extend([
                Instruction::Mov(op, reg!(AX), Quadword),
                Instruction::Push(reg!(AX)),
            ]);
        }
        Instruction::Push(op @ Operand::Imm(i)) if !fits_in_i32(i) => {
            acc.extend([
                Instruction::Mov(op, reg!(R10), Quadword),
                Instruction::Push(reg!(R10)),
            ]);
        }
        other => {
            acc.push(other);
        }
    }
}
