mod label_loops;
mod resolve_labels;
mod resolve_switches;
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
    BreakOutsideLoop,
    #[error("continue statement outside of loop")]
    ContinueOutsideLoop,
    #[error("case label outside of switch")]
    CaseOutsideLoop,
    #[error("default label outside of switch")]
    DefaultOutsideLoop,
    #[error("non-constant case value: {0:?}")]
    NonConstCase(Exp),
    #[error("duplicate case value: {0:?}")]
    DuplicateCaseValue(i32),
    #[error("multiple default statements found in switch")]
    DuplicateDefault,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program) -> ValidationResult<Program> {
    let passes = [
        resolve_variables::resolve_program,
        resolve_labels::resolve_program,
        label_loops::label_program,
        resolve_switches::resolve_program,
    ];

    passes.iter().try_fold(program, |p, pass| pass(p))
}
