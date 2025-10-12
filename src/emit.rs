use crate::models::assembly::*;

pub fn emit(program: Program) -> String {
    let mut acc = String::new();
    emit_program(program, &mut acc);
    acc
}

fn emit_program(program: Program, acc: &mut String) {
    match program {
        Program::Program(function) => emit_function(function, acc),
    }
    acc.push_str("    .section .note.GNU-stack,\"\",@progbits\n")
}

fn emit_function(function: Function, acc: &mut String) {
    match function {
        Function::Function(name, instructions) => {
            acc.push_str(&format!(
                "    .global {name}\n{name}:\n    pushq %rbp\n    movq %rsp, %rbp\n"
            ));
            emit_instructions(instructions, acc)
        }
    }
}

fn emit_instructions(instructions: Vec<Instruction>, acc: &mut String) {
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
        };
        acc.push_str(&fixed);
    }
}
