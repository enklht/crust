use super::assembly::AsmType;
use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type {
    #[display("int")]
    Int,
    #[display("long")]
    Long,
    #[display("uint")]
    UInt,
    #[display("ulong")]
    ULong,
    #[display("double")]
    Double,
    #[display("*{_0}")]
    Pointer(Box<Type>),
    #[display("fn{_0:?} -> {_1}")]
    Func(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn size(&self) -> Option<usize> {
        use Type::*;
        match self {
            Int | UInt => Some(32),
            Long | ULong | Pointer(_) => Some(64),
            _ => None,
        }
    }

    pub fn is_signed(&self) -> bool {
        use Type::*;
        match self {
            Int | Long | Double => true,
            UInt | ULong | Pointer(_) => false,
            _ => unreachable!(),
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Long | Type::UInt | Type::ULong | Type::Double
        )
    }

    pub fn to_asm_type(&self) -> AsmType {
        use Type::*;
        match self {
            Int | UInt => AsmType::Longword,
            Long | ULong | Pointer(_) => AsmType::Quadword,
            Double => AsmType::Double,
            _ => unreachable!(),
        }
    }
}
