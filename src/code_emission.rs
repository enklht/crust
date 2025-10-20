use crate::models::assembly::*;
use crate::models::symbol::Symbols;
use std::fmt::Write;

pub fn emit(program: Program<Plain>, symbols: &Symbols) -> String {
    let mut acc = String::new();
    emit_program(program, &mut acc, symbols).unwrap();
    acc
}

fn emit_program(
    program: Program<Plain>,
    acc: &mut impl Write,
    symbols: &Symbols,
) -> std::fmt::Result {
    let Program::Program(top_levels) = program;
    for top_level in top_levels {
        emit_top_level(top_level, acc, symbols)?;
    }
    writeln!(acc, "    .section .note.GNU-stack,\"\",@progbits")
}

fn emit_top_level(
    top_level: TopLevel<Plain>,
    acc: &mut impl Write,
    symbols: &Symbols,
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
        TopLevel::StaticVar(name, global, init) => {
            if global {
                writeln!(acc, "    .globl {name}")?;
            }
            writeln!(acc, "    .data")?;
            writeln!(acc, "    .balign 4")?;
            writeln!(acc, "{name}:")?;
            if init == 0 {
                writeln!(acc, "    .zero 4")
            } else {
                writeln!(acc, "    .long {init}")
            }
        }
    }
}

fn emit_instructions(
    instructions: Vec<Instruction>,
    acc: &mut impl Write,
    symbols: &Symbols,
) -> std::fmt::Result {
    for instruction in instructions {
        match instruction {
            Instruction::Mov(src, dst) => writeln!(acc, "    movl {src}, {dst}")?,
            Instruction::Unary(op, operand) => writeln!(acc, "    {op} {operand}")?,
            Instruction::Binary(op, src, dst) => writeln!(acc, "    {op} {src}, {dst}")?,
            Instruction::Cmp(src, dst) => writeln!(acc, "    cmpl {src}, {dst}")?,
            Instruction::Idiv(op) => writeln!(acc, "    idivl {op}")?,
            Instruction::Cdq => writeln!(acc, "    cdq")?,
            Instruction::Shl(dst) => writeln!(acc, "    sall %cl, {dst}")?,
            Instruction::Shr(dst) => writeln!(acc, "    sarl %cl, {dst}")?,
            Instruction::Jmp(label) => writeln!(acc, "    jmp .L{label}")?,
            Instruction::JmpCC(code, label) => writeln!(acc, "    j{code} .L{label}")?,
            Instruction::SetCC(code, op) => writeln!(acc, "    set{code} {}", op.one_byte())?,
            Instruction::Label(label) => writeln!(acc, ".L{label}:")?,
            Instruction::AllocateStack(size) => writeln!(acc, "    subq ${size}, %rsp")?,
            Instruction::DeallocateStack(size) => writeln!(acc, "    addq ${size}, %rsp")?,
            Instruction::Push(op) => writeln!(acc, "    pushq {}", op.eight_byte())?,
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

