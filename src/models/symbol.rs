use super::typ::Type;
use crate::models::ast::Exp;
use crate::models::constant::Const;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<String, Symbol>;

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

impl InitialValue {
    pub fn from_exp(init: &Exp, typ: &Type) -> Option<InitialValue> {
        let i = match init {
            Exp::Const(c) => match c {
                Const::Int(i) => *i as i64,
                Const::Long(i) => *i,
                Const::UInt(i) => *i as i64,
                Const::ULong(i) => *i as i64,
            },
            _ => return None,
        };

        match typ {
            Type::Int => Some(InitialValue::Initial(StaticInit::Int(i as i32))),
            Type::Long => Some(InitialValue::Initial(StaticInit::Long(i))),
            Type::UInt => Some(InitialValue::Initial(StaticInit::UInt(i as u32))),
            Type::ULong => Some(InitialValue::Initial(StaticInit::ULong(i as u64))),
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
}

impl StaticInit {
    pub fn alignment(&self) -> usize {
        use StaticInit::*;
        match self {
            Int(_) | UInt(_) => 4,
            Long(_) | ULong(_) => 8,
        }
    }
}
