use derive_more::Display;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<String, Symbol>;

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

pub enum InitialValue {
    Tentative,
    Initial(i32),
    NoInitializer,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("long")]
    Long,
    #[display("Fn")]
    Func(Vec<Type>, Box<Type>),
}
