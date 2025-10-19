use derive_more::Display;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<String, Symbol>;

pub enum Symbol {
    Func {
        arity: usize,
        defined: bool,
        global: bool,
    },
    StaticVar {
        type_: VarType,
        init: InitialValue,
        global: bool,
    },
    LocalVar {
        type_: VarType,
    },
}

pub enum InitialValue {
    Tentative,
    Initial(i32),
    NoInitializer,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum VarType {
    #[display("Int")]
    Int,
}
