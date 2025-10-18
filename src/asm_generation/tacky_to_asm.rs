use crate::models::assembly::*;
use crate::models::tacky;
use crate::models::tacky::BinaryOp as BinOp;

const ARG_REGISTERS: [Reg; 6] = [Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];

pub fn gen_program(program: tacky::Program) -> Program<Plain> {
    match program {
        tacky::Program::Program(func_defs) => {
            Program::Program(func_defs.into_iter().map(gen_func_def).collect())
        }
    }
}

fn gen_func_def(function: tacky::FuncDef) -> FuncDef<Plain> {
    match function {
        tacky::FuncDef::FuncDef(name, params, body) => {
            let mut asm_instructions = Vec::new();
            let (register_params, stack_params) = &params.split_at(params.len().min(6));

            for (arg, reg) in register_params.iter().zip(ARG_REGISTERS) {
                asm_instructions.push(Instruction::Mov(
                    Operand::Reg(reg),
                    Operand::Pseudo(arg.clone()),
                ));
            }

            for (i, arg) in stack_params.iter().enumerate() {
                asm_instructions.push(Instruction::Mov(
                    Operand::Stack((i as isize + 2) * 8),
                    Operand::Pseudo(arg.clone()),
                ))
            }

            asm_instructions.extend(gen_instructions(body));

            FuncDef::Function(name, asm_instructions, ())
        }
    }
}

macro_rules! reg {
    ($reg: ident) => {
        Operand::Reg(Reg::$reg)
    };
}

fn gen_instructions(instructions: Vec<tacky::Instruction>) -> Vec<Instruction> {
    let mut asm_instructions = Vec::new();
    for instruction in instructions {
        match instruction {
            tacky::Instruction::Return(val) => {
                asm_instructions
                    .extend([Instruction::Mov(gen_val(val), reg!(AX)), Instruction::Ret]);
            }
            tacky::Instruction::Unary(tacky::UnaryOp::Not, src, dst) => {
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(src)),
                    Instruction::Mov(Operand::Imm(0), dst.clone()),
                    Instruction::SetCC(CondCode::E, dst),
                ]);
            }
            tacky::Instruction::Unary(op, src, dst) => {
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Mov(gen_val(src), dst.clone()),
                    Instruction::Unary(gen_unop(op), dst),
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
                        Instruction::Mov(gen_val(src1), dst.clone()),
                        Instruction::Binary(gen_binop(op), gen_val(src2), dst),
                    ]);
                }
                BinOp::Divide => {
                    asm_instructions.extend([
                        Instruction::Mov(gen_val(src1), reg!(AX)),
                        Instruction::Cdq,
                        Instruction::Idiv(gen_val(src2)),
                        Instruction::Mov(reg!(AX), gen_val(dst)),
                    ]);
                }
                BinOp::Remainder => {
                    asm_instructions.extend([
                        Instruction::Mov(gen_val(src1), reg!(AX)),
                        Instruction::Cdq,
                        Instruction::Idiv(gen_val(src2)),
                        Instruction::Mov(reg!(DX), gen_val(dst)),
                    ]);
                }
                BinOp::LShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        Instruction::Mov(gen_val(src1), dst.clone()),
                        Instruction::Mov(gen_val(src2), reg!(CX)),
                        Instruction::Shl(dst),
                    ]);
                }
                BinOp::RShift => {
                    let dst = gen_val(dst);
                    asm_instructions.extend([
                        Instruction::Mov(gen_val(src1), dst.clone()),
                        Instruction::Mov(gen_val(src2), reg!(CX)),
                        Instruction::Shr(dst),
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
                        Instruction::Cmp(gen_val(src2), gen_val(src1)),
                        Instruction::Mov(Operand::Imm(0), dst.clone()),
                        Instruction::SetCC(gen_rel(op), dst),
                    ]);
                }
            },
            tacky::Instruction::Jump(target) => {
                asm_instructions.push(Instruction::Jmp(target));
            }
            tacky::Instruction::JumpIfZero(val, target) => {
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(val)),
                    Instruction::JmpCC(CondCode::E, target),
                ]);
            }
            tacky::Instruction::JumpIfNotZero(val, target) => {
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(val)),
                    Instruction::JmpCC(CondCode::NE, target),
                ]);
            }
            tacky::Instruction::Copy(src, dst) => {
                asm_instructions.push(Instruction::Mov(gen_val(src), gen_val(dst)));
            }
            tacky::Instruction::Label(label) => {
                asm_instructions.push(Instruction::Label(label));
            }
            tacky::Instruction::FuncCall(name, args, dst) => {
                let (register_args, stack_args) = &args.split_at(args.len().min(6));

                let stack_padding = if stack_args.len() % 2 == 1 {
                    asm_instructions.push(Instruction::AllocateStack(8));
                    8
                } else {
                    0
                };

                for (arg, reg) in register_args.iter().zip(ARG_REGISTERS) {
                    asm_instructions
                        .push(Instruction::Mov(gen_val(arg.clone()), Operand::Reg(reg)));
                }

                for arg in stack_args.iter().rev() {
                    asm_instructions.push(Instruction::Push(gen_val(arg.clone())));
                }

                asm_instructions.push(Instruction::Call(name));

                let bytes_to_remove = 8 * stack_args.len() + stack_padding;
                if bytes_to_remove > 0 {
                    asm_instructions.push(Instruction::DeallocateStack(bytes_to_remove));
                }

                asm_instructions.push(Instruction::Mov(reg!(AX), gen_val(dst)));
            }
        }
    }
    asm_instructions
}

fn gen_val(val: tacky::Val) -> Operand {
    match val {
        tacky::Val::Const(i) => Operand::Imm(i),
        tacky::Val::Var(ident) => Operand::Pseudo(ident),
    }
}

fn gen_unop(op: tacky::UnaryOp) -> UnaryOp {
    match op {
        tacky::UnaryOp::Complement => UnaryOp::Not,
        tacky::UnaryOp::Negate => UnaryOp::Neg,
        _ => todo!(),
    }
}

fn gen_binop(op: BinOp) -> BinaryOp {
    match op {
        BinOp::Add => BinaryOp::Add,
        BinOp::Subtract => BinaryOp::Subtract,
        BinOp::Multiply => BinaryOp::Multiply,
        BinOp::BitAnd => BinaryOp::BitAnd,
        BinOp::BitOr => BinaryOp::BitOr,
        BinOp::BitXor => BinaryOp::BitXor,
        _ => unreachable!(),
    }
}

fn gen_rel(op: BinOp) -> CondCode {
    match op {
        BinOp::Equal => CondCode::E,
        BinOp::NotEqual => CondCode::NE,
        BinOp::LessThan => CondCode::L,
        BinOp::LessOrEqual => CondCode::LE,
        BinOp::GreaterThan => CondCode::G,
        BinOp::GreaterOrEqual => CondCode::GE,
        _ => unreachable!(),
    }
}
