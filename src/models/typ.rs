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
    #[display("")]
    Array(Box<Type>, usize),
}

impl Type {
    pub fn size(&self) -> Option<usize> {
        use Type::*;
        match self {
            Int | UInt => Some(4),
            Long | ULong | Double | Pointer(_) => Some(8),
            Array(element_type, size) => element_type.size()?.checked_mul(*size),
            _ => None,
        }
    }

    pub fn alignment(&self) -> Option<usize> {
        use Type::*;
        match self {
            Int | UInt => Some(4),
            Long | ULong | Double | Pointer(_) => Some(8),
            Array(element_type, _) => {
                if self.size()? >= 16 {
                    Some(16)
                } else {
                    element_type.alignment()
                }
            }
            _ => None,
        }
    }

    pub fn pointer_scale(&self) -> usize {
        let Type::Pointer(element_type) = &self else {
            unreachable!()
        };
        element_type.size().unwrap()
    }

    pub fn is_signed(&self) -> bool {
        use Type::*;
        match self {
            Int | Long => true,
            UInt | ULong | Pointer(_) | Double => false,
            _ => unreachable!(),
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int | Type::Long | Type::UInt | Type::ULong)
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Long | Type::UInt | Type::ULong | Type::Double
        )
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }

    pub fn to_asm_type(&self) -> AsmType {
        use Type::*;
        match self {
            Int | UInt => AsmType::Longword,
            Long | ULong | Pointer(_) => AsmType::Quadword,
            Double => AsmType::Double,
            Array(_, _) => AsmType::ByteArray(self.size().unwrap(), self.alignment().unwrap()),
            _ => unreachable!(),
        }
    }
}
