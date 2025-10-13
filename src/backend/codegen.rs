use crate::models::assembly as asm;
use crate::models::tacky;
use crate::models::tacky::BinaryOp as BinOp;

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

macro_rules! reg {
    ($reg: ident) => {
        asm::Operand::Reg(asm::Reg::$reg)
    };
}

fn gen_instructions(instructions: Vec<tacky::Instruction>) -> Vec<asm::Instruction> {
    let mut asm_instructions = Vec::new();
    for instruction in instructions {
        match instruction {
            tacky::Instruction::Return(val) => {
                asm_instructions.extend([
                    asm::Instruction::Mov(gen_val(val), reg!(AX)),
                    asm::Instruction::Ret,
                ]);
            }
            tacky::Instruction::Unary(tacky::UnaryOp::Not, src, dst) => {
                let dst = gen_val(dst);
                asm_instructions.extend([
                    asm::Instruction::Cmp(asm::Operand::Imm(0), gen_val(src)),
                    asm::Instruction::Mov(asm::Operand::Imm(0), dst.clone()),
                    asm::Instruction::SetCC(asm::CondCode::E, dst),
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
                        asm::Instruction::Mov(gen_val(src1), reg!(AX)),
                        asm::Instruction::Cdq,
                        asm::Instruction::Idiv(gen_val(src2)),
                        asm::Instruction::Mov(reg!(AX), gen_val(dst)),
                    ]);
                }
                BinOp::Remainder => {
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), reg!(AX)),
                        asm::Instruction::Cdq,
                        asm::Instruction::Idiv(gen_val(src2)),
                        asm::Instruction::Mov(reg!(DX), gen_val(dst)),
                    ]);
                }
                BinOp::LShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Mov(gen_val(src2), reg!(CX)),
                        asm::Instruction::Shl(dst),
                    ]);
                }
                BinOp::RShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Mov(gen_val(src1), dst.clone()),
                        asm::Instruction::Mov(gen_val(src2), reg!(CX)),
                        asm::Instruction::Shr(dst),
                    ]);
                }
                BinOp::Equal
                | BinOp::NotEqual
                | BinOp::LessThan
                | BinOp::LessOrEqual
                | BinOp::GreaterThan
                | BinOp::GreaterOrEqual => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        asm::Instruction::Cmp(gen_val(src2), gen_val(src1)),
                        asm::Instruction::Mov(asm::Operand::Imm(0), dst.clone()),
                        asm::Instruction::SetCC(gen_rel(op), dst),
                    ]);
                }
            },
            tacky::Instruction::Jump(target) => {
                asm_instructions.push(asm::Instruction::Jmp(target));
            }
            tacky::Instruction::JumpIfZero(val, target) => {
                asm_instructions.extend([
                    asm::Instruction::Cmp(asm::Operand::Imm(0), gen_val(val)),
                    asm::Instruction::JmpCC(asm::CondCode::E, target),
                ]);
            }
            tacky::Instruction::JumpIfNotZero(val, target) => {
                asm_instructions.extend([
                    asm::Instruction::Cmp(asm::Operand::Imm(0), gen_val(val)),
                    asm::Instruction::JmpCC(asm::CondCode::NE, target),
                ]);
            }
            tacky::Instruction::Copy(src, dst) => {
                asm_instructions.push(asm::Instruction::Mov(gen_val(src), gen_val(dst)));
            }
            tacky::Instruction::Label(label) => {
                asm_instructions.push(asm::Instruction::Label(label));
            }
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
        _ => todo!(),
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

fn gen_rel(op: BinOp) -> asm::CondCode {
    match op {
        BinOp::Equal => asm::CondCode::E,
        BinOp::NotEqual => asm::CondCode::NE,
        BinOp::LessThan => asm::CondCode::L,
        BinOp::LessOrEqual => asm::CondCode::LE,
        BinOp::GreaterThan => asm::CondCode::G,
        BinOp::GreaterOrEqual => asm::CondCode::GE,
        _ => unreachable!(),
    }
}
