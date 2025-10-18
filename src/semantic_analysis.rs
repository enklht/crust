mod label_loops;
mod resolve_ident;
mod resolve_labels;
mod resolve_switches;
mod type_check;

use crate::models::ast::*;
use crate::models::types::Symbols;
use crate::models::types::Type;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    // invalid variable declaration
    #[error("Variable '{0}' has already been declared in this scope")]
    DuplicateVarDecl(String),
    #[error("Cannot find variable '{0}' in this scope")]
    UndefinedVariable(String),
    #[error("Expression '{0:?}' cannot be used as an lvalue")]
    InvalidLvalue(Exp),
    // invalid function declaration
    #[error("Function '{0}' has already been declared")]
    DuplicateFuncDecl(String),
    #[error("Cannot find function '{0}' in this scope")]
    UndefinedFunction(String),
    #[error("Function '{0}' cannot be defined inside another function")]
    NestedFuncDef(String),
    #[error("Function '{0}' is previously defined as {1}, now defined as {2}")]
    IncompatibleFuncDecl(String, Type, Type),
    // invalid label
    #[error("Cannot find label '{0}' in this scope")]
    UndefinedLabel(String),
    #[error("Label '{0}' has already been defined")]
    DuplicateLabel(String),
    // invalid loop label
    #[error("'break' statement can only be used within a loop")]
    BreakOutsideLoop,
    #[error("'continue' statement can only be used within a loop")]
    ContinueOutsideLoop,
    // invalid switch label
    #[error("'case' statement can only appear inside a switch")]
    CaseOutsideSwitch,
    #[error("Duplicate case value {0:?} in switch statement")]
    DuplicateCaseValue(i32),
    #[error("Case value must be a constant expression, found: {0:?}")]
    NonConstCase(Exp),
    #[error("'default' statement can only appear inside a switch")]
    DefaultOutsideSwitch,
    #[error("Switch statement cannot have multiple 'default' cases")]
    DuplicateDefault,
    // invalid variable
    #[error("'{0}' is a function and cannot be used as a variable")]
    FunctionAsVariable(String),
    // invalid function call
    #[error("'{0}' is a variable and cannot be called as a function")]
    VariableAsFunction(String),
    #[error("Function '{0}' expects {1} arguments but was called with {2} arguments")]
    WrongArity(String, usize, usize),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(program: Program<Plain>) -> ValidationResult<(Program<SwitchResolved>, Symbols)> {
    let passes = [
        resolve_ident::resolve_program,
        resolve_labels::resolve_program,
        label_loops::label_program,
    ];

    let program = passes.iter().try_fold(program, |p, pass| pass(p))?;
    let program = resolve_switches::resolve_program(program)?;

    let symbols = type_check::check_program(&program)?;

    Ok((program, symbols))
}
