use super::reg;
use crate::models::assembly::*;
use crate::models::ast::Const;
use crate::models::symbol::{Symbol, Symbols};
use crate::models::tacky;
use crate::models::tacky::BinaryOp as BinOp;
use crate::models::typ::Type;

const ARG_REGISTERS: [Reg; 6] = [Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];

fn type_of_var(name: &str, symbols: &Symbols) -> AssemblyType {
    match symbols.get(name).unwrap() {
        Symbol::StaticVar { typ, .. } | Symbol::LocalVar { typ } => match typ {
            Type::Int => AssemblyType::Longword,
            Type::Long => AssemblyType::Quadword,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn type_of_val(val: &tacky::Val, symbols: &Symbols) -> AssemblyType {
    match val {
        tacky::Val::Const(Const::Int(_)) => AssemblyType::Longword,
        tacky::Val::Const(Const::Long(_)) => AssemblyType::Quadword,
        tacky::Val::Var(name) => type_of_var(name, symbols),
    }
}

pub fn gen_program(program: tacky::Program, symbols: &Symbols) -> Program {
    match program {
        tacky::Program::Program(top_levels) => Program::Program(
            top_levels
                .into_iter()
                .map(|top_level| gen_top_level(top_level, symbols))
                .collect(),
        ),
    }
}

fn gen_top_level(function: tacky::TopLevel, symbols: &Symbols) -> TopLevel {
    match function {
        tacky::TopLevel::Function(name, global, params, body) => {
            let mut asm_instructions = Vec::new();
            let (register_params, stack_params) = params.split_at(params.len().min(6));

            for (arg, reg) in register_params.iter().zip(ARG_REGISTERS) {
                asm_instructions.push(Instruction::Mov(
                    Operand::Reg(reg),
                    Operand::Pseudo(arg.clone()),
                    type_of_var(arg, symbols),
                ));
            }

            for (i, arg) in stack_params.iter().enumerate() {
                asm_instructions.push(Instruction::Mov(
                    Operand::Stack((i as isize + 2) * 8),
                    Operand::Pseudo(arg.clone()),
                    type_of_var(arg, symbols),
                ))
            }

            asm_instructions.extend(gen_instructions(body, symbols));

            TopLevel::Function(name, global, asm_instructions, None)
        }
        tacky::TopLevel::StaticVar(name, global, init) => {
            TopLevel::StaticVar(name, global, init.alignment(), init)
        }
    }
}

fn gen_instructions(instructions: Vec<tacky::Instruction>, symbols: &Symbols) -> Vec<Instruction> {
    let mut asm_instructions = Vec::new();
    for instruction in instructions {
        use tacky::Instruction::*;
        match instruction {
            Return(val) => {
                let asm_type = type_of_val(&val, symbols);
                asm_instructions.extend([
                    Instruction::Mov(gen_val(val), reg!(AX), asm_type),
                    Instruction::Ret,
                ]);
            }
            Unary(tacky::UnaryOp::Not, src, dst) => {
                let src_type = type_of_val(&src, symbols);
                let dst_type = type_of_val(&dst, symbols);
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(src), src_type),
                    Instruction::Mov(Operand::Imm(0), dst.clone(), dst_type),
                    Instruction::SetCC(CondCode::E, dst),
                ]);
            }
            Unary(op, src, dst) => {
                let asm_type = type_of_val(&src, symbols);
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Mov(gen_val(src), dst.clone(), asm_type),
                    Instruction::Unary(gen_unop(op), dst, asm_type),
                ]);
            }
            Binary(op, src1, src2, dst) => {
                let src1_type = type_of_val(&src1, symbols);
                let dst_type = type_of_val(&dst, symbols);
                let dst = gen_val(dst);
                match op {
                    BinOp::Add
                    | BinOp::Subtract
                    | BinOp::Multiply
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXor => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), dst.clone(), src1_type),
                            Instruction::Binary(gen_binop(op), gen_val(src2), dst, src1_type),
                        ]);
                    }
                    BinOp::Divide => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Cdq(src1_type),
                            Instruction::Idiv(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(AX), dst, src1_type),
                        ]);
                    }
                    BinOp::Remainder => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Cdq(src1_type),
                            Instruction::Idiv(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(DX), dst, src1_type),
                        ]);
                    }
                    BinOp::LShift => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), dst.clone(), src1_type),
                            Instruction::Mov(gen_val(src2), reg!(CX), src1_type),
                            Instruction::Shl(dst, src1_type),
                        ]);
                    }
                    BinOp::RShift => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), dst.clone(), src1_type),
                            Instruction::Mov(gen_val(src2), reg!(CX), src1_type),
                            Instruction::Shr(dst, src1_type),
                        ]);
                    }
                    BinOp::Equal
                    | BinOp::NotEqual
                    | BinOp::LessThan
                    | BinOp::LessOrEqual
                    | BinOp::GreaterThan
                    | BinOp::GreaterOrEqual => {
                        asm_instructions.extend([
                            Instruction::Cmp(gen_val(src2), gen_val(src1), src1_type),
                            Instruction::Mov(Operand::Imm(0), dst.clone(), dst_type),
                            Instruction::SetCC(gen_rel(op), dst),
                        ]);
                    }
                }
            }
            Jump(target) => {
                asm_instructions.push(Instruction::Jmp(target));
            }
            JumpIfZero(cond, target) => {
                let cond_type = type_of_val(&cond, symbols);
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(cond), cond_type),
                    Instruction::JmpCC(CondCode::E, target),
                ]);
            }
            JumpIfNotZero(cond, target) => {
                let cond_type = type_of_val(&cond, symbols);
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(cond), cond_type),
                    Instruction::JmpCC(CondCode::NE, target),
                ]);
            }
            Copy(src, dst) => {
                let src_type = type_of_val(&src, symbols);
                asm_instructions.push(Instruction::Mov(gen_val(src), gen_val(dst), src_type));
            }
            Label(label) => {
                asm_instructions.push(Instruction::Label(label));
            }
            FuncCall(name, args, dst) => {
                let (register_args, stack_args) = args.split_at(args.len().min(6));

                let stack_padding = if stack_args.len() % 2 == 1 {
                    asm_instructions.push(Instruction::Binary(
                        BinaryOp::Subtract,
                        Operand::Imm(8),
                        reg!(SP),
                        AssemblyType::Quadword,
                    ));
                    8
                } else {
                    0
                };

                for (arg, reg) in register_args.iter().zip(ARG_REGISTERS) {
                    asm_instructions.push(Instruction::Mov(
                        gen_val(arg.clone()),
                        Operand::Reg(reg),
                        type_of_val(arg, symbols),
                    ));
                }

                for arg in stack_args.iter().rev() {
                    let asm_type = type_of_val(arg, symbols);
                    let arg = gen_val(arg.clone());
                    if matches!(arg, Operand::Imm(_) | Operand::Reg(_))
                        || asm_type == AssemblyType::Quadword
                    {
                        asm_instructions.push(Instruction::Push(arg))
                    } else {
                        asm_instructions.extend([
                            Instruction::Mov(arg.clone(), reg!(AX), AssemblyType::Longword),
                            Instruction::Push(reg!(AX)),
                        ])
                    }
                }

                asm_instructions.push(Instruction::Call(name));

                let bytes_to_remove = 8 * stack_args.len() + stack_padding;
                if bytes_to_remove > 0 {
                    asm_instructions.push(Instruction::Binary(
                        BinaryOp::Add,
                        Operand::Imm(bytes_to_remove as i64),
                        reg!(SP),
                        AssemblyType::Quadword,
                    ));
                }

                let dst_type = type_of_val(&dst, symbols);
                asm_instructions.push(Instruction::Mov(reg!(AX), gen_val(dst), dst_type));
            }
            SignExtend(src, dst) => {
                asm_instructions.push(Instruction::Movsx(gen_val(src), gen_val(dst)));
            }
            Truncate(src, dst) => asm_instructions.push(Instruction::Mov(
                gen_val(src),
                gen_val(dst),
                AssemblyType::Longword,
            )),
        }
    }
    asm_instructions
}

fn gen_val(val: tacky::Val) -> Operand {
    match val {
        tacky::Val::Const(Const::Int(i)) => Operand::Imm(i as i64),
        tacky::Val::Const(Const::Long(i)) => Operand::Imm(i),
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
