use crate::models::assembly as asm;
use crate::models::ast;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("invalid codegen")]
    CodegenError,
}

pub fn codegen(program: ast::Program) -> Result<asm::Program, CodegenError> {
    match program {
        ast::Program::Function(function) => Ok(asm::Program {
            function: codegen_function(function)?,
        }),
    }
}

fn codegen_function(function: ast::Function) -> Result<asm::Function, CodegenError> {
    match function {
        ast::Function::Function {
            name: ast::Ident::Ident(name),
            body,
        } => Ok(asm::Function {
            name,
            instruction: codegen_stmt(body)?,
        }),
    }
}

fn codegen_stmt(stmt: ast::Stmt) -> Result<Vec<asm::Instruction>, CodegenError> {
    match stmt {
        ast::Stmt::Return(ast::Expr::Int(ast::Int::Const(i))) => Ok(vec![
            asm::Instruction::Mov {
                src: asm::Operand::Imm(i),
                dst: asm::Operand::Register,
            },
            asm::Instruction::Ret,
        ]),
    }
}
