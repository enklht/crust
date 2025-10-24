use super::reg;
use crate::models::assembly::*;
use crate::models::symbol::Symbols;
use crate::models::tacky;
use crate::models::tacky::BinaryOp as BinOp;
use crate::models::typ::Type;

const ARG_REGISTERS: [Reg; 6] = [Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];

fn type_of_var(name: &str, symbols: &Symbols) -> Type {
    symbols.get(name).unwrap().typ().clone()
}

fn type_of_val(val: &tacky::Val, symbols: &Symbols) -> Type {
    match val {
        tacky::Val::Const(c) => c.typ().clone(),
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
                    type_of_var(arg, symbols).to_asm_type(),
                ));
            }

            for (i, arg) in stack_params.iter().enumerate() {
                asm_instructions.push(Instruction::Mov(
                    Operand::Stack((i as isize + 2) * 8),
                    Operand::Pseudo(arg.clone()),
                    type_of_var(arg, symbols).to_asm_type(),
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
                let asm_type = type_of_val(&val, symbols).to_asm_type();
                asm_instructions.extend([
                    Instruction::Mov(gen_val(val), reg!(AX), asm_type),
                    Instruction::Ret,
                ]);
            }
            Unary(tacky::UnaryOp::LogicalNot, src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(src), src_type),
                    Instruction::Mov(Operand::Imm(0), dst.clone(), AsmType::Longword),
                    Instruction::SetCC(CondCode::E, dst),
                ]);
            }
            Unary(op, src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                let dst = gen_val(dst);
                asm_instructions.extend([
                    Instruction::Mov(gen_val(src), dst.clone(), src_type),
                    Instruction::Unary(op, dst, src_type),
                ]);
            }
            Binary(op, src1, src2, dst) => {
                let src1_type = type_of_val(&src1, symbols);
                let (src1_type, signed) = (src1_type.to_asm_type(), src1_type.is_signed());
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
                            Instruction::Binary(op, gen_val(src2), dst, src1_type),
                        ]);
                    }
                    BinOp::Divide if signed => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Cdq(src1_type),
                            Instruction::Idiv(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(AX), dst, src1_type),
                        ]);
                    }
                    BinOp::Divide if !signed => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Mov(Operand::Imm(0), reg!(DX), src1_type),
                            Instruction::Div(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(AX), dst, src1_type),
                        ]);
                    }
                    BinOp::Remainder if signed => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Cdq(src1_type),
                            Instruction::Idiv(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(DX), dst, src1_type),
                        ]);
                    }
                    BinOp::Remainder if !signed => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), reg!(AX), src1_type),
                            Instruction::Mov(Operand::Imm(0), reg!(DX), src1_type),
                            Instruction::Div(gen_val(src2), src1_type),
                            Instruction::Mov(reg!(DX), dst, src1_type),
                        ]);
                    }
                    BinOp::LShift | BinOp::RShift => {
                        asm_instructions.extend([
                            Instruction::Mov(gen_val(src1), dst.clone(), src1_type),
                            Instruction::Mov(gen_val(src2), reg!(CX), src1_type),
                            Instruction::Shift(gen_shift_op(op, signed), dst, src1_type),
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
                            Instruction::Mov(Operand::Imm(0), dst.clone(), AsmType::Longword),
                            Instruction::SetCC(gen_rel(op, signed), dst),
                        ]);
                    }
                    _ => unreachable!(),
                }
            }
            Jump(target) => {
                asm_instructions.push(Instruction::Jmp(target));
            }
            JumpIfZero(cond, target) => {
                let cond_type = type_of_val(&cond, symbols).to_asm_type();
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(cond), cond_type),
                    Instruction::JmpCC(CondCode::E, target),
                ]);
            }
            JumpIfNotZero(cond, target) => {
                let cond_type = type_of_val(&cond, symbols).to_asm_type();
                asm_instructions.extend([
                    Instruction::Cmp(Operand::Imm(0), gen_val(cond), cond_type),
                    Instruction::JmpCC(CondCode::NE, target),
                ]);
            }
            Copy(src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
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
                        AsmType::Quadword,
                    ));
                    8
                } else {
                    0
                };

                for (arg, reg) in register_args.iter().zip(ARG_REGISTERS) {
                    asm_instructions.push(Instruction::Mov(
                        gen_val(arg.clone()),
                        Operand::Reg(reg),
                        type_of_val(arg, symbols).to_asm_type(),
                    ));
                }

                for arg in stack_args.iter().rev() {
                    let asm_type = type_of_val(arg, symbols).to_asm_type();
                    let arg = gen_val(arg.clone());
                    if matches!(arg, Operand::Imm(_) | Operand::Reg(_))
                        || asm_type == AsmType::Quadword
                    {
                        asm_instructions.push(Instruction::Push(arg))
                    } else {
                        asm_instructions.extend([
                            Instruction::Mov(arg.clone(), reg!(AX), AsmType::Longword),
                            Instruction::Push(reg!(AX)),
                        ])
                    }
                }

                asm_instructions.push(Instruction::Call(name));

                let bytes_to_remove = 8 * stack_args.len() + stack_padding;
                if bytes_to_remove > 0 {
                    asm_instructions.push(Instruction::Binary(
                        BinaryOp::Add,
                        Operand::Imm(bytes_to_remove as u64),
                        reg!(SP),
                        AsmType::Quadword,
                    ));
                }

                let dst_type = type_of_val(&dst, symbols).to_asm_type();
                asm_instructions.push(Instruction::Mov(reg!(AX), gen_val(dst), dst_type));
            }
            SignExtend(src, dst) => {
                asm_instructions.push(Instruction::Movsx(gen_val(src), gen_val(dst)));
            }
            ZeroExtend(src, dst) => {
                asm_instructions.push(Instruction::MovZeroExtend(gen_val(src), gen_val(dst)));
            }
            Truncate(src, dst) => asm_instructions.push(Instruction::Mov(
                gen_val(src),
                gen_val(dst),
                AsmType::Longword,
            )),
        }
    }
    asm_instructions
}

fn gen_val(val: tacky::Val) -> Operand {
    match val {
        tacky::Val::Const(c) => Operand::Imm(c.to_u64()),
        tacky::Val::Var(ident) => Operand::Pseudo(ident),
    }
}

fn gen_shift_op(op: BinOp, signed: bool) -> ShiftOp {
    match op {
        BinOp::LShift if signed => ShiftOp::Sal,
        BinOp::LShift if !signed => ShiftOp::Shl,
        BinOp::RShift if signed => ShiftOp::Sar,
        BinOp::RShift if !signed => ShiftOp::Shr,
        _ => unreachable!(),
    }
}

fn gen_rel(op: BinOp, signed: bool) -> CondCode {
    match (op, signed) {
        (BinOp::Equal, _) => CondCode::E,
        (BinOp::NotEqual, _) => CondCode::NE,
        (BinOp::LessThan, true) => CondCode::L,
        (BinOp::LessThan, false) => CondCode::B,
        (BinOp::LessOrEqual, true) => CondCode::LE,
        (BinOp::LessOrEqual, false) => CondCode::BE,
        (BinOp::GreaterThan, true) => CondCode::G,
        (BinOp::GreaterThan, false) => CondCode::A,
        (BinOp::GreaterOrEqual, true) => CondCode::GE,
        (BinOp::GreaterOrEqual, false) => CondCode::AE,
        _ => unreachable!(),
    }
}
