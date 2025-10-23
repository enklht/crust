use super::assembly::AsmType;
use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("long")]
    Long,
    #[display("Fn{_0:?} -> {_1}")]
    Func(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn to_asm_type(&self) -> AsmType {
        match self {
            Type::Int => AsmType::Longword,
            Type::Long => AsmType::Quadword,
            _ => unreachable!(),
        }
    }
}
