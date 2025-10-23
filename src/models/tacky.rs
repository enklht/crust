pub use super::constant::Const;
pub use super::operator::*;
use super::symbol::StaticInit;

#[derive(Debug)]
pub enum Program {
    Program(Vec<TopLevel>),
}

pub enum TopLevel {
    Function(String, bool, Vec<String>, Vec<Instruction>),
    StaticVar(String, bool, StaticInit),
}

#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    SignExtend(Val, Val),
    Truncate(Val, Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val),
    Copy(Val, Val),
    Jump(String),
    JumpIfZero(Val, String),
    JumpIfNotZero(Val, String),
    Label(String),
    FuncCall(String, Vec<Val>, Val),
}

#[derive(Debug, Clone)]
pub enum Val {
    Const(Const),
    Var(String),
}

impl std::fmt::Debug for TopLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TopLevel::Function(name, _global, params, instructions) => {
                writeln!(f, "\nname: {name}")?;
                writeln!(f, "params: {params:?}")?;
                write!(
                    f,
                    "instructions: {:?}",
                    crate::utils::PrettyVec(instructions)
                )
            }
            TopLevel::StaticVar(name, global, init) => {
                writeln!(f, "\nstatic {name} := {init:?}")?;
                write!(f, "global: {global}")
            }
        }
    }
}
