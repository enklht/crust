use crate::models::assembly as asm;
use crate::models::ast;

pub fn codegen(program: ast::Program) -> asm::Program {
    match program {
        ast::Program::Function(function) => asm::Program::Program(codegen_function(function)),
    }
}

fn codegen_function(function: ast::Function) -> asm::Function {
    match function {
        ast::Function::Function { name, body } => asm::Function::Function {
            name,
            instructions: codegen_stmt(body),
        },
    }
}

fn codegen_stmt(stmt: ast::Stmt) -> Vec<asm::Instruction> {
    match stmt {
        ast::Stmt::Return(ast::Expr::Const(i)) => vec![
            asm::Instruction::Mov {
                src: asm::Operand::Imm(i),
                dst: asm::Operand::Register,
            },
            asm::Instruction::Ret,
        ],
        _ => todo!(),
    }
}
