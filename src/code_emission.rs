use crate::models::asm_symbol::AsmSymbols;
use crate::models::assembly::*;
use crate::models::symbol::StaticInit;
use std::fmt::Write;

pub fn emit(program: Program, symbols: &AsmSymbols) -> String {
    let mut acc = String::new();
    emit_program(program, &mut acc, symbols).unwrap();
    acc
}

fn emit_program(program: Program, acc: &mut impl Write, symbols: &AsmSymbols) -> std::fmt::Result {
    let Program::Program(top_levels) = program;
    for top_level in top_levels {
        emit_top_level(top_level, acc, symbols)?;
    }
    writeln!(acc, "    .section .note.GNU-stack,\"\",@progbits")
}

fn emit_top_level(
    top_level: TopLevel,
    acc: &mut impl Write,
    symbols: &AsmSymbols,
) -> std::fmt::Result {
    match top_level {
        TopLevel::Function(name, global, instructions, _) => {
            if global {
                writeln!(acc, "    .globl {name}")?;
            }
            writeln!(acc, "    .text")?;
            writeln!(acc, "{name}:")?;
            writeln!(acc, "    pushq %rbp")?;
            writeln!(acc, "    movq %rsp, %rbp")?;
            emit_instructions(instructions, acc, symbols)
        }
        TopLevel::StaticVar(name, global, alignment, init) => {
            use StaticInit::*;
            if global {
                writeln!(acc, "    .globl {name}")?;
            }
            match init {
                Int(0) | Long(0) | UInt(0) | ULong(0) => writeln!(acc, "    .bss")?,
                _ => writeln!(acc, "    .data")?,
            }
            writeln!(acc, "    .balign {alignment}")?;
            writeln!(acc, "{name}:")?;
            match init {
                Int(0) | UInt(0) => writeln!(acc, "    .zero 4"),
                Long(0) | ULong(0) => writeln!(acc, "    .zero 8"),
                Int(i) => writeln!(acc, "    .long {i}"),
                UInt(i) => writeln!(acc, "    .long {i}"),
                Long(i) => writeln!(acc, "    .quad {i}"),
                ULong(i) => writeln!(acc, "    .quad {i}"),
                Double(i) => writeln!(acc, "    .quad {}", i.to_bits()),
            }
        }
        TopLevel::StaticConst(name, alignment, init) => {
            use StaticInit::*;
            writeln!(acc, "    .section .rodata")?;
            writeln!(acc, "    .balign {alignment}")?;
            writeln!(acc, "{name}:")?;
            match init {
                Int(0) | UInt(0) => writeln!(acc, "    .zero 4"),
                Long(0) | ULong(0) => writeln!(acc, "    .zero 8"),
                Int(i) => writeln!(acc, "    .long {i}"),
                UInt(i) => writeln!(acc, "    .long {i}"),
                Long(i) => writeln!(acc, "    .quad {i}"),
                ULong(i) => writeln!(acc, "    .quad {i}"),
                Double(i) => {
                    writeln!(acc, "    # {i}")?;
                    writeln!(acc, "    .quad {}", i.to_bits())
                }
            }
        }
    }
}

fn emit_instructions(
    instructions: Vec<Instruction>,
    acc: &mut impl Write,
    symbols: &AsmSymbols,
) -> std::fmt::Result {
    use AsmType::*;
    for instruction in instructions {
        match instruction {
            Instruction::Mov(src, dst, typ) => {
                writeln!(acc, "    mov{typ} {}, {}", fmt(&src, typ), fmt(&dst, typ))?
            }
            Instruction::Movsx(src, dst) => {
                writeln!(
                    acc,
                    "    movslq {}, {}",
                    fmt(&src, Longword),
                    fmt(&dst, Quadword)
                )?;
            }
            Instruction::Cvtsi2sd(src, dst, typ) => {
                writeln!(
                    acc,
                    "    cvtsi2sd{typ} {}, {}",
                    fmt(&src, typ),
                    fmt(&dst, Double)
                )?;
            }
            Instruction::Cvttsd2si(src, dst, typ) => {
                writeln!(
                    acc,
                    "    cvttsd2si{typ} {}, {}",
                    fmt(&src, Double),
                    fmt(&dst, typ)
                )?;
            }
            Instruction::Unary(op, operand, typ) => {
                writeln!(acc, "    {op}{typ} {}", fmt(&operand, typ))?
            }
            Instruction::Binary(op, src, dst, typ) => {
                use BinaryOp::*;
                match (op, typ) {
                    (Multiply, Double) => {
                        writeln!(acc, "    mulsd {}, {}", fmt(&src, typ), fmt(&dst, typ))?;
                    }
                    (BitAnd | BitOr | BitXor, Double) => {
                        writeln!(acc, "    {op}pd {}, {}", fmt(&src, typ), fmt(&dst, typ))?;
                    }
                    (Sal | Sar | Shl | Shr, _) => {
                        writeln!(acc, "    {op}{typ} {}, {}", fmt(&src, Byte), fmt(&dst, typ))?;
                    }
                    _ => {
                        writeln!(acc, "    {op}{typ} {}, {}", fmt(&src, typ), fmt(&dst, typ))?;
                    }
                }
            }
            Instruction::Cmp(src, dst, typ) => {
                match typ {
                    Double => writeln!(acc, "    ucomisd {}, {}", fmt(&src, typ), fmt(&dst, typ))?,
                    _ => writeln!(acc, "    cmp{typ} {}, {}", fmt(&src, typ), fmt(&dst, typ))?,
                };
            }
            Instruction::Idiv(op, typ) => writeln!(acc, "    idiv{typ} {}", fmt(&op, typ))?,
            Instruction::Div(op, typ) => writeln!(acc, "    div{typ} {}", fmt(&op, typ))?,
            Instruction::Cdq(AsmType::Longword) => writeln!(acc, "    cdq")?,
            Instruction::Cdq(AsmType::Quadword) => writeln!(acc, "    cqo")?,
            Instruction::Jmp(label) => writeln!(acc, "    jmp .L{label}")?,
            Instruction::JmpCC(code, label) => writeln!(acc, "    j{code} .L{label}")?,
            Instruction::SetCC(code, op) => writeln!(acc, "    set{code} {}", fmt(&op, Byte))?,
            Instruction::Label(label) => writeln!(acc, ".L{label}:")?,
            Instruction::Push(op) => writeln!(acc, "    pushq {}", fmt(&op, Quadword))?,
            Instruction::Call(name) => {
                let defined = match symbols.get(&name) {
                    Some(crate::models::asm_symbol::AsmSymbol::FunEntry(defined)) => *defined,
                    _ => false,
                };

                if defined {
                    writeln!(acc, "    call {name}")?;
                } else {
                    writeln!(acc, "    call {name}@PLT")?;
                }
            }
            Instruction::Ret => {
                writeln!(acc, "    movq %rbp, %rsp")?;
                writeln!(acc, "    popq %rbp")?;
                writeln!(acc, "    ret")?;
            }
            Instruction::MovZeroExtend(_, _) => unreachable!(),
            _ => todo!(),
        }
    }
    Ok(())
}
