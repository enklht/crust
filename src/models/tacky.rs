pub use super::constant::Const;
pub use super::operator::*;
use super::symbol::StaticInit;
use crate::utils::Ident;

#[derive(Debug)]
pub enum Program {
    Program(Vec<TopLevel>),
}

pub enum TopLevel {
    Function(Ident, bool, Vec<Ident>, Vec<Instruction>),
    StaticVar(Ident, bool, Vec<StaticInit>),
}

#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    SignExtend(Val, Val),
    Truncate(Val, Val),
    ZeroExtend(Val, Val),
    DoubleToInt(Val, Val),
    DoubleToUInt(Val, Val),
    IntToDouble(Val, Val),
    UIntToDouble(Val, Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val),
    Copy(Val, Val),
    GetAddress(Val, Val),
    Load(Val, Val),
    Store(Val, Val),
    AddPtr(Val, Val, usize, Val),
    CopyToOffset(Val, Ident, usize),
    Jump(Ident),
    JumpIfZero(Val, Ident),
    JumpIfNotZero(Val, Ident),
    Label(Ident),
    FuncCall(Ident, Vec<Val>, Val),
}

#[derive(Debug, Clone, Copy)]
pub enum Val {
    Const(Const),
    Var(Ident),
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
