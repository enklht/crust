mod resolve;

use crate::models::ast::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Semantic error: duplicate variable definition {0}")]
    DuplicateVariableDefinition(String),
    #[error("Semantic error: lvalue of an assignement must be a variable, found {0:?}")]
    InvalidLvalue(Expr),
    #[error("Semantic error: variable {0} is not defined")]
    UndefinedVariable(String),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program) -> ValidationResult<Program> {
    resolve::resolve_program(program)
}
