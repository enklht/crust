use crate::models::assembly as asm;
use crate::models::{tacky, tacky::BinaryOp as BinOp};

pub fn gen_program(program: tacky::Program) -> asm::Program {
    match program {
        tacky::Program::Program(function) => asm::Program::Program(gen_function(function)),
    }
}

fn gen_function(function: tacky::Function) -> asm::Function {
    match function {
        tacky::Function::Function(name, body) => {
            asm::Function::Function(name, gen_instructions(body))
        }
    }
}

fn gen_instructions(instructions: Vec<tacky::Instruction>) -> Vec<asm::Instruction> {
    let mut asm_instructions = Vec::new();
    for instruction in instructions {
        match instruction {
            tacky::Instruction::Return(val) => {
                asm_instructions.extend([
                    asm::Instruction::Mov(gen_val(val), asm::Operand::Reg(asm::Reg::AX)),
                    asm::Instruction::Ret,
                ]);
            }
            tacky::Instruction::Unary(op, src, dst) => {
                let dst = gen_val(dst);
                asm_instructions.extend([
                    asm::Instruction::Mov(gen_val(src), dst.clone()),
                    asm::Instruction::Unary(gen_unop(op), dst),
                ]);
            }
            tacky::Instruction::Binary(op, src1, src2, dst) => match op {
                BinOp::Add
                | BinOp::Subtract
                | BinOp::Multiply
                | BinOp::BitAnd
                | BinOp::BitOr
                | BinOp::BitXor => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Binary(gen_binop(op), gen_val(src2), dst),
                    ]);
                }
                BinOp::Divide => {
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), asm::Operand::Reg(asm::Reg::AX)),
                        asm::Instruction::Cdq,
                        asm::Instruction::Idiv(gen_val(src2)),
                        asm::Instruction::Mov(asm::Operand::Reg(asm::Reg::AX), gen_val(dst)),
                    ]);
                }
                BinOp::LShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Mov(gen_val(src2), asm::Operand::Reg(asm::Reg::CX)),
                        asm::Instruction::Shl(dst),
                    ]);
                }
                BinOp::RShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Mov(gen_val(src2), asm::Operand::Reg(asm::Reg::CX)),
                        asm::Instruction::Shr(dst),
                    ]);
                }
                BinOp::Remainder => {
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), asm::Operand::Reg(asm::Reg::AX)),
                        asm::Instruction::Cdq,
                        asm::Instruction::Idiv(gen_val(src2)),
                        asm::Instruction::Mov(asm::Operand::Reg(asm::Reg::DX), gen_val(dst)),
                    ]);
                }
            },
        }
    }
    asm_instructions
}

fn gen_val(val: tacky::Val) -> asm::Operand {
    match val {
        tacky::Val::Const(i) => asm::Operand::Imm(i),
        tacky::Val::Var(ident) => asm::Operand::Pseudo(ident),
    }
}

fn gen_unop(op: tacky::UnaryOp) -> asm::UnaryOp {
    match op {
        tacky::UnaryOp::Complement => asm::UnaryOp::Not,
        tacky::UnaryOp::Negate => asm::UnaryOp::Neg,
    }
}

fn gen_binop(op: BinOp) -> asm::BinaryOp {
    match op {
        BinOp::Add => asm::BinaryOp::Add,
        BinOp::Subtract => asm::BinaryOp::Subtract,
        BinOp::Multiply => asm::BinaryOp::Multiply,
        BinOp::BitAnd => asm::BinaryOp::BitAnd,
        BinOp::BitOr => asm::BinaryOp::BitOr,
        BinOp::BitXor => asm::BinaryOp::BitXor,
        _ => unreachable!(),
    }
}
