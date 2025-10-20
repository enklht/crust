use crate::models::assembly::*;
use crate::models::symbol::Symbols;

pub fn emit(program: Program<Plain>, symbols: &Symbols) -> String {
    let mut acc = String::new();
    emit_program(program, &mut acc, symbols);
    acc
}

fn emit_program(program: Program<Plain>, acc: &mut String, symbols: &Symbols) {
    match program {
        Program::Program(func_defs) => {
            for func_def in func_defs {
                emit_func_def(func_def, acc, symbols);
            }
        }
    }
    acc.push_str("    .section .note.GNU-stack,\"\",@progbits\n")
}

fn emit_func_def(func_def: TopLevel<Plain>, acc: &mut String, symbols: &Symbols) {
    match func_def {
        TopLevel::Function(name, global, instructions, _) => {
            if global {
                acc.push_str(&format!("    .globl {name}\n"))
            };
            acc.push_str(&format!(
                "    .text\n{name}:\n    pushq %rbp\n    movq %rsp, %rbp\n"
            ));
            emit_instructions(instructions, acc, symbols)
        }
        TopLevel::StaticVar(name, global, init) => {
            if global {
                acc.push_str(&format!("    .globl {name}\n"))
            };
            acc.push_str(&format!("    .data\n    .balign 4\n{name}:\n"));
            if init == 0 {
                acc.push_str("    .zero 4\n");
            } else {
                acc.push_str(&format!("    .long {init}\n"));
            }
        }
    }
}

fn emit_instructions(instructions: Vec<Instruction>, acc: &mut String, symbols: &Symbols) {
    for instruction in instructions {
        let fixed = match instruction {
            Instruction::Mov(src, dst) => format!("    movl {src}, {dst}\n"),
            Instruction::Ret => "    movq %rbp, %rsp\n    popq %rbp\n    ret\n".to_string(),
            Instruction::Unary(operator, operand) => format!("    {operator} {operand}\n"),
            Instruction::Binary(operator, src, dst) => format!("    {operator} {src}, {dst}\n"),
            Instruction::Idiv(op) => format!("    idivl {op}\n"),
            Instruction::Cdq => "    cdq\n".to_string(),
            Instruction::Shl(dst) => format!("    sall %cl, {dst}\n"),
            Instruction::Shr(dst) => format!("    sarl %cl, {dst}\n"),
            Instruction::AllocateStack(size) => format!("    subq ${size}, %rsp\n"),
            Instruction::Cmp(src, dst) => format!("    cmpl {src}, {dst}\n"),
            Instruction::Jmp(label) => format!("    jmp .L{label}\n"),
            Instruction::JmpCC(code, label) => format!("    j{code} .L{label}\n"),
            Instruction::SetCC(code, op) => format!("    set{code} {}\n", op.one_byte()),
            Instruction::Label(label) => format!(".L{label}:\n"),
            Instruction::DeallocateStack(size) => format!("    addq ${size}, %rsp\n"),
            Instruction::Push(op) => format!("    pushq {}\n", op.eight_byte()),
            Instruction::Call(name) => {
                if symbols.contains_key(&name) {
                    format!("    call {name}\n")
                } else {
                    format!("    call {name}@PLT")
                }
            }
        };
        acc.push_str(&fixed);
    }
}
