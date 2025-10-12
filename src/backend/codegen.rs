use crate::models::assembly as asm;
use crate::models::tacky;

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
                tacky::BinaryOp::Add | tacky::BinaryOp::Subtract | tacky::BinaryOp::Multiply => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Binary(gen_binop(op), gen_val(src2), dst),
                    ]);
                }
                tacky::BinaryOp::Divide => {
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), asm::Operand::Reg(asm::Reg::AX)),
                        asm::Instruction::Cdq,
                        asm::Instruction::Idiv(gen_val(src2)),
                        asm::Instruction::Mov(asm::Operand::Reg(asm::Reg::AX), gen_val(dst)),
                    ]);
                }
                tacky::BinaryOp::Remainder => {
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

fn gen_binop(op: tacky::BinaryOp) -> asm::BinaryOp {
    match op {
        tacky::BinaryOp::Add => asm::BinaryOp::Add,
        tacky::BinaryOp::Subtract => asm::BinaryOp::Sub,
        tacky::BinaryOp::Multiply => asm::BinaryOp::Mult,
        _ => unreachable!(),
    }
}
