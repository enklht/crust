use super::typ::Type;
use crate::models::ast::ExpKind;
use crate::models::ast::Initializer;
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
    Initial(Vec<StaticInit>),
    NoInitializer,
}

fn from_f64(i: f64, typ: &Type) -> StaticInit {
    use StaticInit::*;
    match typ {
        Type::Int => Int(i as i32),
        Type::Long => Long(i as i64),
        Type::UInt => UInt(i as u32),
        Type::ULong => ULong(i as u64),
        Type::Double => Double(i),
        _ => unreachable!(),
    }
}

fn to_f64(c: Const) -> StaticInit {
    use StaticInit::*;
    match c {
        Const::Int(i) => Double(i as f64),
        Const::Long(i) => Double(i as f64),
        Const::UInt(i) => Double(i as f64),
        Const::ULong(i) => Double(i as f64),
        Const::Double(i) => Double(i),
    }
}

fn from_none(typ: &Type) -> StaticInit {
    match typ {
        Type::Int => StaticInit::Int(0),
        Type::Long => StaticInit::Long(0),
        Type::UInt => StaticInit::UInt(0),
        Type::ULong => StaticInit::ULong(0),
        Type::Double => StaticInit::Double(0.),
        Type::Pointer(_) => StaticInit::ULong(0),
        Type::Array(_, _) => StaticInit::ZeroInit(typ.size().expect("array size fits in usize")),
        _ => unreachable!(),
    }
}

impl InitialValue {
    pub fn from_none(typ: &Type) -> InitialValue {
        InitialValue::Initial(vec![from_none(typ)])
    }

    pub fn from_initializer(init: &Initializer, typ: &Type) -> Option<InitialValue> {
        fn inner(init: &Initializer, typ: &Type) -> Option<Vec<StaticInit>> {
            match (typ, init) {
                (Type::Array(element_type, size), Initializer::Compound(inits)) => {
                    if inits.len() > *size {
                        return None;
                    }

                    let mut flattened: Vec<StaticInit> =
                        inits.iter().try_fold(Vec::new(), |mut acc, init| {
                            acc.extend(inner(init, element_type)?);
                            Some(acc)
                        })?;

                    let zero_bytes = (*size - inits.len()) * element_type.size()?;
                    if zero_bytes > 0 {
                        flattened.push(StaticInit::ZeroInit(zero_bytes));
                    }

                    Some(flattened)
                }
                (Type::Array(_, _), Initializer::Single(_)) => None,
                (_, Initializer::Single(exp)) => {
                    if typ.is_pointer() && exp.is_null_pointer_constant() {
                        return Some(vec![from_none(typ)]);
                    }

                    let c = match &exp.kind {
                        ExpKind::Const(c) if !typ.is_pointer() => c,
                        _ => return None,
                    };

                    let i = match c {
                        _ if typ == &Type::Double => return Some(vec![to_f64(*c)]),
                        Const::Double(i) => return Some(vec![from_f64(*i, typ)]),
                        Const::Int(i) => *i as u64,
                        Const::Long(i) => *i as u64,
                        Const::UInt(i) => *i as u64,
                        Const::ULong(i) => *i,
                    };

                    match typ {
                        Type::Int => Some(vec![StaticInit::Int(i as i32)]),
                        Type::Long => Some(vec![StaticInit::Long(i as i64)]),
                        Type::UInt => Some(vec![StaticInit::UInt(i as u32)]),
                        Type::ULong => Some(vec![StaticInit::ULong(i)]),
                        _ => unreachable!(),
                    }
                }
                _ => None,
            }
        }

        Some(InitialValue::Initial(inner(init, typ)?))
    }
}

#[derive(Debug, Clone)]
pub enum StaticInit {
    Int(i32),
    Long(i64),
    UInt(u32),
    ULong(u64),
    Double(f64),
    ZeroInit(usize),
}

impl StaticInit {
    pub fn alignment(&self) -> usize {
        use StaticInit::*;
        match self {
            Int(_) | UInt(_) => 4,
            Long(_) | ULong(_) | Double(_) => 8,
            _ => todo!(),
        }
    }
}
