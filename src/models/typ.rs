use super::assembly::AsmType;
use derive_more::Display;
use std::cmp::Ordering;

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
    pub fn size(&self) -> usize {
        use Type::*;
        match self {
            Int | UInt => 32,
            Long | ULong => 64,
            _ => unreachable!(),
        }
    }

    pub fn is_signed(&self) -> bool {
        use Type::*;
        match self {
            Int | Long | Double => true,
            UInt | ULong => false,
            _ => unreachable!(),
        }
    }

    pub fn to_asm_type(&self) -> AsmType {
        use Type::*;
        match self {
            Int | UInt => AsmType::Longword,
            Long | ULong => AsmType::Quadword,
            Double => AsmType::Double,
            _ => unreachable!(),
        }
    }

    pub fn common_type_with(&self, other: &Type) -> Type {
        if self == other {
            self.clone()
        } else if self == &Type::Double || other == &Type::Double {
            Type::Double
        } else {
            match self.size().cmp(&other.size()) {
                Ordering::Equal if self.is_signed() => other.clone(),
                Ordering::Equal => self.clone(),
                Ordering::Greater => self.clone(),
                Ordering::Less => other.clone(),
            }
        }
    }
}
