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
        Function::Function(name, instructions) => acc.push_str(&format!(
            "    .global {name}
{name}:
    pushq %rbp
    movq %rsp, %rbp
{}
",
            instructions_to_string(instructions)
        )),
    }
}

fn instructions_to_string(instructions: Vec<Instruction>) -> String {
    instructions
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
