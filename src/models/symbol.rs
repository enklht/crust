use crate::models::ast::Exp;
use derive_more::Display;
use rustc_hash::FxHashMap;

pub type Types = FxHashMap<Exp, Type>;
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
    pub fn typ(&self) -> Type {
        match self {
            Symbol::Func { typ, .. } => typ.clone(),
            Symbol::StaticVar { typ, .. } => typ.clone(),
            Symbol::LocalVar { typ, .. } => typ.clone(),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("long")]
    Long,
    #[display("Fn")]
    Func(Vec<Type>, Box<Type>),
}
