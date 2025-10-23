use super::typ::Type;
use crate::models::assembly::AsmType;
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

    pub fn asm_typ(&self) -> AsmType {
        match self {
            Symbol::Func { typ, .. } => typ.to_asm_type(),
            Symbol::StaticVar { typ, .. } => typ.to_asm_type(),
            Symbol::LocalVar { typ, .. } => typ.to_asm_type(),
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
            },
            _ => return None,
        };

        match typ {
            Type::Int => Some(InitialValue::Initial(StaticInit::Int(i as i32))),
            Type::Long => Some(InitialValue::Initial(StaticInit::Long(i))),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StaticInit {
    Int(i32),
    Long(i64),
}

impl StaticInit {
    pub fn alignment(&self) -> usize {
        match self {
            StaticInit::Int(_) => 4,
            StaticInit::Long(_) => 8,
        }
    }
}
