mod label_loops;
mod resolve_labels;
mod resolve_variables;

use crate::models::ast::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("duplicate variable definition {0}")]
    DuplicateVariableDefinition(String),
    #[error("lvalue of an assignement must be a variable, found {0:?}")]
    InvalidLvalue(Exp),
    #[error("variable {0} is not defined")]
    UndefinedVariable(String),
    #[error("duplicate label definition: {0}")]
    DuplicateLabelDefinition(String),
    #[error("label {0} is not defined")]
    UndefinedLabel(String),
    #[error("break statement outside of loop")]
    ContinueOutsideLoop,
    #[error("continue statement outside of loop")]
    BreakOutsideLoop,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program) -> ValidationResult<Program> {
    let program = resolve_variables::resolve_program(program)?;
    let program = resolve_labels::resolve_program(program)?;
    let program = label_loops::label_program(program)?;
    Ok(program)
}
