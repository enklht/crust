use super::typ::Type;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Const {
    Int(i32),
    Long(i64),
    UInt(u32),
    ULong(u64),
    Double(f64),
}

impl std::cmp::Eq for Const {}

impl core::hash::Hash for Const {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Const::*;
        match self {
            Int(i) => i.hash(state),
            Long(i) => i.hash(state),
            UInt(i) => i.hash(state),
            ULong(i) => i.hash(state),
            Double(_) => unreachable!("double is unhashable"),
        }
    }
}

impl Const {
    pub fn convert_to(self, typ: &Type) -> Const {
        use Const::*;
        let i = match self {
            Int(i) => i as u64,
            Long(i) => i as u64,
            UInt(i) => i as u64,
            ULong(i) => i,
            Double(i) => i.to_bits(),
        };

        match typ {
            Type::Int => Const::Int(i as i32),
            Type::Long => Const::Long(i as i64),
            Type::UInt => Const::UInt(i as u32),
            Type::ULong => Const::ULong(i),
            Type::Double => Const::Double(f64::from_bits(i)),
            _ => unreachable!(),
        }
    }

    pub fn typ(&self) -> &Type {
        use Const::*;
        match self {
            Int(_) => &Type::Int,
            Long(_) => &Type::Long,
            UInt(_) => &Type::UInt,
            ULong(_) => &Type::ULong,
            Double(_) => &Type::Double,
        }
    }

    pub fn to_u64(self) -> u64 {
        use Const::*;
        match self {
            Int(i) => i as u64,
            Long(i) => i as u64,
            UInt(i) => i as u64,
            ULong(i) => i,
            Double(i) => i.to_bits(),
        }
    }
}
