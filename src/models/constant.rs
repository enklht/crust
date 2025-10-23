use crate::models::assembly::AsmType;

use super::typ::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Const {
    Int(i32),
    Long(i64),
    UInt(u32),
    ULong(u64),
}

impl Const {
    pub fn convert_to(self, typ: &Type) -> Const {
        match (self, typ) {
            (Const::Int(i), Type::Int) => Const::Int(i),
            (Const::Long(i), Type::Int) => Const::Int(i as i32),
            (Const::Int(i), Type::Long) => Const::Long(i as i64),
            (Const::Long(i), Type::Long) => Const::Long(i),
            _ => unreachable!(),
        }
    }

    pub fn typ(&self) -> &Type {
        match self {
            Const::Int(_) => &Type::Int,
            Const::Long(_) => &Type::Long,
            Const::UInt(_) => &Type::UInt,
            Const::ULong(_) => &Type::ULong,
        }
    }

    pub fn asm_typ(&self) -> AsmType {
        match self {
            Const::Int(_) => AsmType::Longword,
            Const::Long(_) => AsmType::Quadword,
            _ => todo!(),
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Const::Int(i) => *i as i64,
            Const::Long(i) => *i,
            Const::UInt(i) => *i as i64,
            Const::ULong(i) => *i as i64,
        }
    }
}
