mod resolve_labels;
mod resolve_variables;

use crate::models::ast::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation error: duplicate variable definition {0}")]
    DuplicateVariableDefinition(String),
    #[error("Validation error: lvalue of an assignement must be a variable, found {0:?}")]
    InvalidLvalue(Exp),
    #[error("Validation error: variable {0} is not defined")]
    UndefinedVariable(String),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program) -> ValidationResult<Program> {
    let program = resolve_variables::resolve_program(program)?;
    let program = resolve_labels::resolve_program(program)?;
    Ok(program)
}
