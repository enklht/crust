use crate::models::assembly::*;

pub fn emit(program: Program) -> String {
    let mut acc = String::new();
    emit_program(program, &mut acc);
    acc
}

fn emit_program(Program { function }: Program, acc: &mut String) {
    emit_function(function, acc);
    acc.push_str("    .section .note.GNU-stack,\"\",@progbits\n")
}

fn emit_function(Function { name, instructions }: Function, acc: &mut String) {
    acc.push_str(&format!(
        "    .global {}
{}:
{}
",
        name,
        name,
        instructions_to_string(instructions)
    ))
}

fn instructions_to_string(instructions: Vec<Instruction>) -> String {
    instructions
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
