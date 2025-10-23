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
            if global {
                writeln!(acc, "    .globl {name}")?;
            }
            match init {
                StaticInit::Int(0) => writeln!(acc, "    .bss")?,
                StaticInit::Long(0) => writeln!(acc, "    .bss")?,
                StaticInit::Int(_) => writeln!(acc, "    .data")?,
                StaticInit::Long(_) => writeln!(acc, "    .data")?,
                _ => todo!(),
            }
            writeln!(acc, "    .balign {alignment}")?;
            writeln!(acc, "{name}:")?;
            match init {
                StaticInit::Int(0) => writeln!(acc, "    .zero 4"),
                StaticInit::Int(i) => writeln!(acc, "    .long {i}"),
                StaticInit::Long(0) => writeln!(acc, "    .zero 8"),
                StaticInit::Long(i) => writeln!(acc, "    .quad {i}"),
                _ => todo!(),
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
            Instruction::Movsx(src, dst) => writeln!(
                acc,
                "    movslq {}, {}",
                fmt(&src, Longword),
                fmt(&dst, Quadword)
            )?,
            Instruction::Unary(op, operand, typ) => {
                writeln!(acc, "    {op}{typ} {}", fmt(&operand, typ))?
            }
            Instruction::Binary(BinaryOp::LShift, _, dst, typ) => {
                writeln!(acc, "    sal{typ} %cl, {}", fmt(&dst, typ))?
            }
            Instruction::Binary(BinaryOp::RShift, _, dst, typ) => {
                writeln!(acc, "    sar{typ} %cl, {}", fmt(&dst, typ))?
            }
            Instruction::Binary(op, src, dst, typ) => {
                writeln!(acc, "    {op}{typ} {}, {}", fmt(&src, typ), fmt(&dst, typ))?
            }
            Instruction::Cmp(src, dst, typ) => {
                writeln!(acc, "    cmp{typ} {}, {}", fmt(&src, typ), fmt(&dst, typ))?
            }
            Instruction::Idiv(op, typ) => writeln!(acc, "    idiv{typ} {}", fmt(&op, typ),)?,
            Instruction::Cdq(AsmType::Byte) => panic!(),
            Instruction::Cdq(AsmType::Longword) => writeln!(acc, "    cdq")?,
            Instruction::Cdq(AsmType::Quadword) => writeln!(acc, "    cqo")?,
            Instruction::Jmp(label) => writeln!(acc, "    jmp .L{label}")?,
            Instruction::JmpCC(code, label) => writeln!(acc, "    j{code} .L{label}")?,
            Instruction::SetCC(code, op) => writeln!(acc, "    set{code} {}", fmt(&op, Byte))?,
            Instruction::Label(label) => writeln!(acc, ".L{label}:")?,
            Instruction::Push(op) => writeln!(acc, "    pushq {}", fmt(&op, Quadword))?,
            Instruction::Call(name) => {
                if symbols.contains_key(&name) {
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
        }
    }
    Ok(())
}
