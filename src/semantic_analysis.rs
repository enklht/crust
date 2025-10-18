mod label_loops;
mod resolve_ident;
mod resolve_labels;
mod resolve_switches;

use crate::models::ast::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    // variable declaration
    #[error("duplicate variable declaration {0}")]
    DuplicateVarDecl(String),
    #[error("variable {0} is not defined")]
    UndefinedVariable(String),
    #[error("invalid lvalue: {0:?}")]
    InvalidLvalue(Exp),
    // function declaration
    #[error("duplicate function declaration {0}")]
    DuplicateFuncDecl(String),
    #[error("function {0} is not defined")]
    UndefinedFunction(String),
    #[error("function {0} is defined inside a function")]
    NestedFuncDef(String),
    // invalid labels
    #[error("label {0} is not defined")]
    UndefinedLabel(String),
    #[error("duplicate label definition: {0}")]
    DuplicateLabel(String),
    // invalid loop label
    #[error("break statement outside of loop")]
    BreakOutsideLoop,
    #[error("continue statement outside of loop")]
    ContinueOutsideLoop,
    // invalid swtich label
    #[error("case label outside of switch")]
    CaseOutsideLoop,
    #[error("duplicate case value: {0:?}")]
    DuplicateCaseValue(i32),
    #[error("non-constant case value: {0:?}")]
    NonConstCase(Exp),
    #[error("default label outside of switch")]
    DefaultOutsideLoop,
    #[error("multiple default statements found in switch")]
    DuplicateDefault,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program) -> ValidationResult<Program> {
    let passes = [
        resolve_ident::resolve_program,
        resolve_labels::resolve_program,
        label_loops::label_program,
        resolve_switches::resolve_program,
    ];

    passes.iter().try_fold(program, |p, pass| pass(p))
}
