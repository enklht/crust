use super::assembly::AssemblyType;
use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("long")]
    Long,
    #[display("Fn{_0:?} -> {_1}")]
    Func(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn to_asm_type(&self) -> AssemblyType {
        match self {
            Type::Int => AssemblyType::Longword,
            Type::Long => AssemblyType::Quadword,
            _ => unreachable!(),
        }
    }
}
