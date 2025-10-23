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
        let i = match self {
            Const::Int(i) => i as i64,
            Const::Long(i) => i,
            Const::UInt(i) => i as i64,
            Const::ULong(i) => i as i64,
        };

        match typ {
            Type::Int => Const::Int(i as i32),
            Type::Long => Const::Long(i),
            Type::UInt => Const::UInt(i as u32),
            Type::ULong => Const::ULong(i as u64),
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

    pub fn to_i64(&self) -> i64 {
        match self {
            Const::Int(i) => *i as i64,
            Const::Long(i) => *i,
            Const::UInt(i) => *i as i64,
            Const::ULong(i) => *i as i64,
        }
    }
}
