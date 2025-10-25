use super::imm;
use super::reg;
use crate::models::assembly::*;
use crate::models::constant::Const;
use crate::models::symbol::*;
use crate::models::tacky;
use crate::models::typ::Type;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

const INT_ARG_REGISTERS: [Reg; 6] = [Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];
const DOUBLE_ARG_REGISTERS: [Reg; 8] = [
    Reg::XMM0,
    Reg::XMM1,
    Reg::XMM2,
    Reg::XMM3,
    Reg::XMM4,
    Reg::XMM5,
    Reg::XMM6,
    Reg::XMM7,
];

type Consts = FxHashMap<u64, (usize, String)>;

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
        tacky::Program::Program(top_levels) => {
            let mut consts = FxHashMap::default();
            let mut top_levels: Vec<_> = top_levels
                .into_iter()
                .map(|top_level| gen_top_level(top_level, symbols, &mut consts))
                .collect();

            top_levels.extend(consts.into_iter().map(|(val, (alignment, label))| {
                TopLevel::StaticConst(label, alignment, StaticInit::Double(f64::from_bits(val)))
            }));

            Program::Program(top_levels)
        }
    }
}

fn gen_top_level(top_level: tacky::TopLevel, symbols: &Symbols, consts: &mut Consts) -> TopLevel {
    match top_level {
        tacky::TopLevel::Function(name, global, params, body) => {
            let mut asm_instructions = Vec::new();
            let (int_reg_params, double_reg_params, stack_params) = classify_params(
                params.into_iter().map(tacky::Val::Var).collect(),
                symbols,
                consts,
            );

            for ((arg, typ), reg) in int_reg_params.into_iter().zip(INT_ARG_REGISTERS) {
                asm_instructions.push(Instruction::Mov(Operand::Reg(reg), arg, typ));
            }

            for ((arg, typ), reg) in double_reg_params.into_iter().zip(DOUBLE_ARG_REGISTERS) {
                asm_instructions.push(Instruction::Mov(Operand::Reg(reg), arg, typ));
            }

            for (i, (arg, typ)) in stack_params.into_iter().enumerate() {
                asm_instructions.push(Instruction::Mov(
                    Operand::Stack((i as isize + 2) * 8),
                    arg,
                    typ,
                ))
            }

            asm_instructions.extend(gen_instructions(body, symbols, consts));

            TopLevel::Function(name, global, asm_instructions, None)
        }
        tacky::TopLevel::StaticVar(name, global, init) => {
            TopLevel::StaticVar(name, global, init.alignment(), init)
        }
    }
}

fn gen_instructions(
    instructions: Vec<tacky::Instruction>,
    symbols: &Symbols,
    consts: &mut Consts,
) -> Vec<Instruction> {
    use AsmType::*;
    use tacky::Instruction::*;

    let mut asm_instructions = Vec::new();

    for instruction in instructions {
        match instruction {
            Return(val) => {
                let asm_type = type_of_val(&val, symbols).to_asm_type();
                let dst = match asm_type {
                    Double => reg!(XMM0),
                    _ => reg!(AX),
                };

                asm_instructions.extend([
                    Instruction::Mov(gen_val(val, consts), dst, asm_type),
                    Instruction::Ret,
                ]);
            }
            SignExtend(src, dst) => {
                asm_instructions.push(Instruction::Movsx(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                ));
            }
            Truncate(src, dst) => {
                asm_instructions.push(Instruction::Mov(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                    Longword,
                ));
            }
            ZeroExtend(src, dst) => {
                asm_instructions.push(Instruction::MovZeroExtend(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                ));
            }
            IntToDouble(src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                asm_instructions.push(Instruction::Cvtsi2sd(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                    src_type,
                ));
            }
            DoubleToInt(src, dst) => {
                let dst_type = type_of_val(&dst, symbols).to_asm_type();
                asm_instructions.push(Instruction::Cvttsd2si(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                    dst_type,
                ));
            }
            UIntToDouble(src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                match src_type {
                    Longword => {
                        asm_instructions.extend([
                            Instruction::MovZeroExtend(gen_val(src, consts), reg!(R8)),
                            Instruction::Cvtsi2sd(reg!(R8), gen_val(dst, consts), Quadword),
                        ]);
                    }
                    Quadword => {
                        let out_of_range_label = make_label("ul2d_out_of_range");
                        let end_label = make_label("ul2d_end");

                        let src = gen_val(src, consts);
                        let dst = gen_val(dst, consts);

                        asm_instructions.extend([
                            Instruction::Cmp(imm!(0), src.clone(), Quadword),
                            Instruction::JmpCC(CondCode::L, out_of_range_label.clone()),
                            Instruction::Cvtsi2sd(src.clone(), dst.clone(), Quadword),
                            Instruction::Jmp(end_label.clone()),
                            Instruction::Label(out_of_range_label),
                            Instruction::Mov(src, reg!(R8), Quadword),
                            Instruction::Mov(reg!(R8), reg!(R9), Quadword),
                            Instruction::Unary(UnaryOp::Shr, reg!(R9), Quadword),
                            Instruction::Binary(BinaryOp::BitAnd, imm!(1), reg!(R8), Quadword),
                            Instruction::Binary(BinaryOp::BitAnd, imm!(1), reg!(R8), Quadword),
                            Instruction::Binary(BinaryOp::BitOr, reg!(R8), reg!(R9), Quadword),
                            Instruction::Cvtsi2sd(reg!(R9), dst.clone(), Quadword),
                            Instruction::Binary(BinaryOp::Add, dst.clone(), dst.clone(), Double),
                            Instruction::Label(end_label),
                        ]);
                    }
                    _ => unreachable!(),
                }
            }
            DoubleToUInt(src, dst) => {
                let dst_type = type_of_val(&dst, symbols).to_asm_type();
                match dst_type {
                    Longword => {
                        asm_instructions.extend([
                            Instruction::Cvttsd2si(gen_val(src, consts), reg!(R8), Quadword),
                            Instruction::Mov(reg!(R8), gen_val(dst, consts), Quadword),
                        ]);
                    }
                    Quadword => {
                        let (_, upper_bound) = consts
                            .entry(9223372036854775808.0f64.to_bits())
                            .or_insert((8, make_label("upper_bound")));
                        let upper_bound = Operand::Data(upper_bound.clone());

                        let out_of_range_label = make_label("d2ul_out_of_range");
                        let end_label = make_label("d2ul_end");

                        let src = gen_val(src, consts);
                        let dst = gen_val(dst, consts);

                        asm_instructions.extend([
                            Instruction::Cmp(upper_bound.clone(), src.clone(), Double),
                            Instruction::JmpCC(CondCode::AE, out_of_range_label.clone()),
                            Instruction::Cvttsd2si(src.clone(), dst.clone(), Quadword),
                            Instruction::Jmp(end_label.clone()),
                            Instruction::Label(out_of_range_label),
                            Instruction::Mov(src, reg!(XMM7), Double),
                            Instruction::Binary(
                                BinaryOp::Subtract,
                                upper_bound,
                                reg!(XMM7),
                                Double,
                            ),
                            Instruction::Cvttsd2si(reg!(XMM7), dst.clone(), Quadword),
                            Instruction::Mov(imm!(i32::MAX as u64), reg!(R8), Quadword),
                            Instruction::Binary(BinaryOp::Add, reg!(R8), dst, Quadword),
                            Instruction::Label(end_label),
                        ]);
                    }
                    _ => unreachable!(),
                }
            }
            Unary(op, src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                let dst_type = type_of_val(&dst, symbols).to_asm_type();
                let src = gen_val(src, consts);
                let dst = gen_val(dst, consts);

                match op {
                    UnaryOp::LogicalNot if src_type == Double => {
                        asm_instructions.extend([
                            Instruction::Binary(BinaryOp::BitXor, reg!(XMM0), reg!(XMM0), Double),
                            Instruction::Cmp(src, reg!(XMM0), Double),
                            Instruction::Mov(imm!(0), dst.clone(), dst_type),
                            Instruction::SetCC(CondCode::E, dst.clone()),
                            Instruction::SetCC(CondCode::NP, reg!(AX)),
                            Instruction::Binary(BinaryOp::BitAnd, reg!(AX), dst, Byte),
                        ]);
                    }
                    UnaryOp::LogicalNot => {
                        asm_instructions.extend([
                            Instruction::Cmp(imm!(0), src, src_type),
                            Instruction::Mov(imm!(0), dst.clone(), Longword),
                            Instruction::SetCC(CondCode::E, dst),
                        ]);
                    }
                    UnaryOp::Negate if src_type == Double => {
                        let (_, neg_zero) = consts
                            .entry((-0.0f64).to_bits())
                            .or_insert((16, make_label("negative_zero")));
                        let neg_zero = Operand::Data(neg_zero.clone());
                        asm_instructions.extend([
                            Instruction::Mov(src, dst.clone(), Double),
                            Instruction::Binary(BinaryOp::BitXor, neg_zero, dst, Double),
                        ]);
                    }
                    _ => {
                        asm_instructions.extend([
                            Instruction::Mov(src, dst.clone(), src_type),
                            Instruction::Unary(op, dst, src_type),
                        ]);
                    }
                }
            }
            Binary(op, src1, src2, dst) => {
                let src1_type = type_of_val(&src1, symbols);
                let (src1_type, signed) = (src1_type.to_asm_type(), src1_type.is_signed());

                let src1 = gen_val(src1, consts);
                let src2 = gen_val(src2, consts);
                let dst = gen_val(dst, consts);

                match op {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor => {
                        asm_instructions.extend([
                            Instruction::Mov(src1, dst.clone(), src1_type),
                            Instruction::Binary(op, src2, dst, src1_type),
                        ]);
                    }
                    BinaryOp::Divide => {
                        if src1_type == Double {
                            asm_instructions.extend([
                                Instruction::Mov(src1, dst.clone(), src1_type),
                                Instruction::Binary(BinaryOp::Divide, src2, dst, src1_type),
                            ]);
                        } else if signed {
                            asm_instructions.extend([
                                Instruction::Mov(src1, reg!(AX), src1_type),
                                Instruction::Cdq(src1_type),
                                Instruction::Idiv(src2, src1_type),
                                Instruction::Mov(reg!(AX), dst, src1_type),
                            ]);
                        } else {
                            asm_instructions.extend([
                                Instruction::Mov(src1, reg!(AX), src1_type),
                                Instruction::Mov(imm!(0), reg!(DX), src1_type),
                                Instruction::Div(src2, src1_type),
                                Instruction::Mov(reg!(AX), dst, src1_type),
                            ]);
                        }
                    }
                    BinaryOp::Remainder => {
                        if signed {
                            asm_instructions.extend([
                                Instruction::Mov(src1, reg!(AX), src1_type),
                                Instruction::Cdq(src1_type),
                                Instruction::Idiv(src2, src1_type),
                                Instruction::Mov(reg!(DX), dst, src1_type),
                            ]);
                        } else {
                            asm_instructions.extend([
                                Instruction::Mov(src1, reg!(AX), src1_type),
                                Instruction::Mov(imm!(0), reg!(DX), src1_type),
                                Instruction::Div(src2, src1_type),
                                Instruction::Mov(reg!(DX), dst, src1_type),
                            ]);
                        }
                    }
                    BinaryOp::LShift | BinaryOp::RShift => {
                        asm_instructions.extend([
                            Instruction::Mov(src1, dst.clone(), src1_type),
                            Instruction::Mov(src2, reg!(CX), src1_type),
                            Instruction::Shift(gen_shift_op(op, signed), dst, src1_type),
                        ]);
                    }
                    BinaryOp::Equal
                    | BinaryOp::LessThan
                    | BinaryOp::LessOrEqual
                    | BinaryOp::GreaterThan
                    | BinaryOp::GreaterOrEqual
                        if src1_type == Double =>
                    {
                        asm_instructions.extend([
                            Instruction::Cmp(src2, src1, src1_type),
                            Instruction::Mov(imm!(0), dst.clone(), Longword),
                            Instruction::SetCC(gen_rel(op, true), dst.clone()),
                            Instruction::SetCC(CondCode::NP, reg!(AX)),
                            Instruction::Binary(BinaryOp::BitAnd, reg!(AX), dst.clone(), Byte),
                        ]);
                    }
                    BinaryOp::NotEqual if src1_type == Double => {
                        asm_instructions.extend([
                            Instruction::Cmp(src2, src1, src1_type),
                            Instruction::Mov(imm!(0), dst.clone(), Longword),
                            Instruction::SetCC(gen_rel(op, true), dst.clone()),
                            Instruction::SetCC(CondCode::P, reg!(AX)),
                            Instruction::Binary(BinaryOp::BitOr, reg!(AX), dst.clone(), Byte),
                        ]);
                    }
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::LessThan
                    | BinaryOp::LessOrEqual
                    | BinaryOp::GreaterThan
                    | BinaryOp::GreaterOrEqual => {
                        asm_instructions.extend([
                            Instruction::Cmp(src2, src1, src1_type),
                            Instruction::Mov(imm!(0), dst.clone(), Longword),
                            Instruction::SetCC(gen_rel(op, !signed), dst.clone()),
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
                let cond = gen_val(cond, consts);
                match cond_type {
                    Double => {
                        asm_instructions.extend([
                            Instruction::Binary(BinaryOp::BitXor, reg!(XMM0), reg!(XMM0), Double),
                            Instruction::Cmp(cond, reg!(XMM0), Double),
                            Instruction::SetCC(CondCode::E, reg!(AX)),
                            Instruction::SetCC(CondCode::NP, reg!(DX)),
                            Instruction::Binary(BinaryOp::BitAnd, reg!(DX), reg!(AX), Byte),
                            Instruction::Cmp(imm!(0), reg!(AX), Byte),
                            Instruction::JmpCC(CondCode::NE, target),
                        ]);
                    }
                    _ => {
                        asm_instructions.extend([
                            Instruction::Cmp(imm!(0), cond, cond_type),
                            Instruction::JmpCC(CondCode::E, target),
                        ]);
                    }
                }
            }
            JumpIfNotZero(cond, target) => {
                let cond_type = type_of_val(&cond, symbols).to_asm_type();
                let cond = gen_val(cond, consts);
                match cond_type {
                    Double => {
                        asm_instructions.extend([
                            Instruction::Binary(BinaryOp::BitXor, reg!(XMM0), reg!(XMM0), Double),
                            Instruction::Cmp(cond, reg!(XMM0), Double),
                            Instruction::SetCC(CondCode::NE, reg!(AX)),
                            Instruction::SetCC(CondCode::P, reg!(DX)),
                            Instruction::Binary(BinaryOp::BitOr, reg!(DX), reg!(AX), Byte),
                            Instruction::Cmp(imm!(0), reg!(AX), Byte),
                            Instruction::JmpCC(CondCode::NE, target),
                        ]);
                    }
                    _ => {
                        asm_instructions.extend([
                            Instruction::Cmp(imm!(0), cond, cond_type),
                            Instruction::JmpCC(CondCode::NE, target),
                        ]);
                    }
                }
            }
            Copy(src, dst) => {
                let src_type = type_of_val(&src, symbols).to_asm_type();
                asm_instructions.push(Instruction::Mov(
                    gen_val(src, consts),
                    gen_val(dst, consts),
                    src_type,
                ));
            }
            Label(label) => {
                asm_instructions.push(Instruction::Label(label));
            }
            FuncCall(name, args, dst) => {
                let (int_reg_args, double_reg_args, stack_args) =
                    classify_params(args, symbols, consts);

                let stack_padding = if stack_args.len() % 2 == 1 {
                    asm_instructions.push(Instruction::Binary(
                        BinaryOp::Subtract,
                        imm!(8),
                        reg!(SP),
                        Quadword,
                    ));
                    8
                } else {
                    0
                };
                let bytes_to_remove = 8 * stack_args.len() + stack_padding;

                for ((arg, typ), reg) in int_reg_args.into_iter().zip(INT_ARG_REGISTERS) {
                    asm_instructions.push(Instruction::Mov(arg, Operand::Reg(reg), typ));
                }

                for ((arg, typ), reg) in double_reg_args.into_iter().zip(DOUBLE_ARG_REGISTERS) {
                    asm_instructions.push(Instruction::Mov(arg, Operand::Reg(reg), typ));
                }

                for (arg, typ) in stack_args.into_iter().rev() {
                    if matches!(arg, Operand::Imm(_) | Operand::Reg(_))
                        || typ == Quadword
                        || typ == Double
                    {
                        asm_instructions.push(Instruction::Push(arg))
                    } else {
                        asm_instructions.extend([
                            Instruction::Mov(arg, reg!(AX), typ),
                            Instruction::Push(reg!(AX)),
                        ])
                    }
                }

                asm_instructions.push(Instruction::Call(name));

                if bytes_to_remove > 0 {
                    asm_instructions.push(Instruction::Binary(
                        BinaryOp::Add,
                        imm!(bytes_to_remove as u64),
                        reg!(SP),
                        Quadword,
                    ));
                }

                let dst_type = type_of_val(&dst, symbols).to_asm_type();
                let dst = gen_val(dst, consts);
                match dst_type {
                    Double => asm_instructions.push(Instruction::Mov(reg!(XMM0), dst, dst_type)),
                    _ => asm_instructions.push(Instruction::Mov(reg!(AX), dst, dst_type)),
                }
            }
        }
    }

    asm_instructions
}

type Typed = (Operand, AsmType);
fn classify_params(
    vals: Vec<tacky::Val>,
    symbols: &Symbols,
    consts: &mut Consts,
) -> (Vec<Typed>, Vec<Typed>, Vec<Typed>) {
    let mut int_reg_params = Vec::new();
    let mut double_reg_params = Vec::new();
    let mut stack_params = Vec::new();

    for val in vals {
        let typ = type_of_val(&val, symbols).to_asm_type();
        let op = gen_val(val, consts);
        let typed = (op, typ);

        if typ == AsmType::Double {
            if double_reg_params.len() < 8 {
                double_reg_params.push(typed);
            } else {
                stack_params.push(typed);
            }
        } else if int_reg_params.len() < 6 {
            int_reg_params.push(typed);
        } else {
            stack_params.push(typed);
        }
    }

    (int_reg_params, double_reg_params, stack_params)
}

fn gen_val(val: tacky::Val, consts: &mut Consts) -> Operand {
    match val {
        tacky::Val::Const(Const::Double(d)) => {
            let (_, label) = consts
                .entry(d.to_bits())
                .or_insert((8, make_label("static_const")));
            Operand::Data(label.clone())
        }
        tacky::Val::Const(c) => imm!(c.to_u64()),
        tacky::Val::Var(ident) => Operand::Pseudo(ident),
    }
}

fn gen_shift_op(op: BinaryOp, signed: bool) -> ShiftOp {
    match op {
        BinaryOp::LShift if signed => ShiftOp::Sal,
        BinaryOp::LShift if !signed => ShiftOp::Shl,
        BinaryOp::RShift if signed => ShiftOp::Sar,
        BinaryOp::RShift if !signed => ShiftOp::Shr,
        _ => unreachable!(),
    }
}

fn gen_rel(op: BinaryOp, unsigned_or_double: bool) -> CondCode {
    use BinaryOp::*;
    match (op, unsigned_or_double) {
        (Equal, _) => CondCode::E,
        (NotEqual, _) => CondCode::NE,
        (LessThan, false) => CondCode::L,
        (LessThan, true) => CondCode::B,
        (LessOrEqual, false) => CondCode::LE,
        (LessOrEqual, true) => CondCode::BE,
        (GreaterThan, false) => CondCode::G,
        (GreaterThan, true) => CondCode::A,
        (GreaterOrEqual, false) => CondCode::GE,
        (GreaterOrEqual, true) => CondCode::AE,
        _ => unreachable!(),
    }
}
