mod block_labelling;
mod ident_resolution;
mod label_resolution;
mod switch_resolution;
mod type_check;

use crate::models::ast::*;
use crate::models::symbol::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0}")]
    Label(String),
    #[error("{0}")]
    FuncDecl(String),
    #[error("{0}")]
    VarDecl(String),
    #[error("{0}")]
    Type(String),
    #[error("{0}")]
    LoopLabel(String),
    #[error("{0}")]
    Switch(String),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(mut program: Program) -> ValidationResult<(Program, Symbols)> {
    ident_resolution::resolve_program(&mut program)?;

    let mut program = label_resolution::resolve_program(program)?;

    let symbols = type_check::check_program(&mut program)?;

    block_labelling::label_program(&mut program)?;
    switch_resolution::resolve_program(&mut program)?;

    Ok((program, symbols))
}
