use super::typ::Type;
use crate::models::ast::Exp;
use crate::models::constant::Const;
use crate::utils::Ident;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<Ident, Symbol>;

#[derive(Debug)]
pub enum Symbol {
    Func {
        typ: Type,
        defined: bool,
        global: bool,
    },
    StaticVar {
        typ: Type,
        init: InitialValue,
        global: bool,
    },
    LocalVar {
        typ: Type,
    },
}

impl Symbol {
    pub fn typ(&self) -> &Type {
        match self {
            Symbol::Func { typ, .. } => typ,
            Symbol::StaticVar { typ, .. } => typ,
            Symbol::LocalVar { typ, .. } => typ,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InitialValue {
    Tentative,
    Initial(StaticInit),
    NoInitializer,
}

fn from_f64(i: f64, typ: &Type) -> InitialValue {
    use InitialValue::Initial;
    use StaticInit::*;
    match typ {
        Type::Int => Initial(Int(i as i32)),
        Type::Long => Initial(Long(i as i64)),
        Type::UInt => Initial(UInt(i as u32)),
        Type::ULong => Initial(ULong(i as u64)),
        Type::Double => Initial(Double(i)),
        _ => unreachable!(),
    }
}

fn to_f64(c: Const) -> InitialValue {
    use InitialValue::Initial;
    use StaticInit::*;
    match c {
        Const::Int(i) => Initial(Double(i as f64)),
        Const::Long(i) => Initial(Double(i as f64)),
        Const::UInt(i) => Initial(Double(i as f64)),
        Const::ULong(i) => Initial(Double(i as f64)),
        Const::Double(i) => Initial(Double(i)),
    }
}

impl InitialValue {
    pub fn from_exp(init: &Exp, typ: &Type) -> Option<InitialValue> {
        let c = match init {
            Exp::Const(c) => c,
            _ => return None,
        };

        let i = match c {
            _ if typ == &Type::Double => return Some(to_f64(*c)),
            Const::Double(i) => return Some(from_f64(*i, typ)),
            Const::Int(i) => *i as u64,
            Const::Long(i) => *i as u64,
            Const::UInt(i) => *i as u64,
            Const::ULong(i) => *i,
        };

        match typ {
            Type::Int => Some(InitialValue::Initial(StaticInit::Int(i as i32))),
            Type::Long => Some(InitialValue::Initial(StaticInit::Long(i as i64))),
            Type::UInt => Some(InitialValue::Initial(StaticInit::UInt(i as u32))),
            Type::ULong => Some(InitialValue::Initial(StaticInit::ULong(i))),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StaticInit {
    Int(i32),
    Long(i64),
    UInt(u32),
    ULong(u64),
    Double(f64),
}

impl StaticInit {
    pub fn alignment(&self) -> usize {
        use StaticInit::*;
        match self {
            Int(_) | UInt(_) => 4,
            Long(_) | ULong(_) | Double(_) => 8,
        }
    }
}
