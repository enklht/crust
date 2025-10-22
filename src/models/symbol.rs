use super::typ::Type;
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

#[derive(Debug, Clone)]
pub enum StaticInit {
    IntInit(i32),
    LongInit(i64),
}

impl StaticInit {
    pub fn alignment(&self) -> usize {
        match self {
            StaticInit::IntInit(_) => 4,
            StaticInit::LongInit(_) => 8,
        }
    }
}
