use super::typ::{TypeId, TypeKind, Types};
use crate::models::ast::ExpKind;
use crate::models::ast::Initializer;
use crate::models::constant::Const;
use crate::utils::Ident;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<Ident, Symbol>;

#[derive(Debug)]
pub enum Symbol {
    Func {
        typ: TypeId,
        defined: bool,
        global: bool,
    },
    StaticVar {
        typ: TypeId,
        init: InitialValue,
        global: bool,
    },
    LocalVar {
        typ: TypeId,
    },
}

impl Symbol {
    pub fn typ(&self) -> TypeId {
        match self {
            Symbol::Func { typ, .. } => *typ,
            Symbol::StaticVar { typ, .. } => *typ,
            Symbol::LocalVar { typ, .. } => *typ,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InitialValue {
    Tentative,
    Initial(Vec<StaticInit>),
    NoInitializer,
}

fn from_f64(i: f64, typ: TypeId, types: &Types) -> StaticInit {
    use StaticInit::*;
    match types.kind(typ) {
        TypeKind::Int => Int(i as i32),
        TypeKind::Long => Long(i as i64),
        TypeKind::UInt => UInt(i as u32),
        TypeKind::ULong => ULong(i as u64),
        TypeKind::Double => Double(i),
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

fn from_none(typ: TypeId, types: &Types) -> StaticInit {
    match types.kind(typ) {
        TypeKind::Int => StaticInit::Int(0),
        TypeKind::Long => StaticInit::Long(0),
        TypeKind::UInt => StaticInit::UInt(0),
        TypeKind::ULong => StaticInit::ULong(0),
        TypeKind::Double => StaticInit::Double(0.),
        TypeKind::Pointer(_) => StaticInit::ULong(0),
        TypeKind::Array(_, _) => {
            StaticInit::ZeroInit(types.size(typ).expect("array size fits in usize"))
        }
        _ => unreachable!(),
    }
}

impl InitialValue {
    pub fn from_none(typ: TypeId, types: &Types) -> InitialValue {
        InitialValue::Initial(vec![from_none(typ, types)])
    }

    pub fn from_initializer(
        init: &Initializer,
        typ: TypeId,
        types: &Types,
    ) -> Option<InitialValue> {
        fn inner(init: &Initializer, typ: TypeId, types: &Types) -> Option<Vec<StaticInit>> {
            match (types.kind(typ), init) {
                (TypeKind::Array(element_type, size), Initializer::Compound(_, inits)) => {
                    if inits.len() > *size {
                        return None;
                    }

                    let mut flattened: Vec<StaticInit> =
                        inits.iter().try_fold(Vec::new(), |mut acc, init| {
                            acc.extend(inner(init, *element_type, types)?);
                            Some(acc)
                        })?;

                    let zero_bytes = (*size - inits.len()) * types.size(*element_type)?;
                    if zero_bytes > 0 {
                        flattened.push(StaticInit::ZeroInit(zero_bytes));
                    }

                    Some(flattened)
                }
                (TypeKind::Array(_, _), Initializer::Single(_)) => None,
                (_, Initializer::Single(exp)) => {
                    if types.is_pointer(typ) && exp.is_null_pointer_constant() {
                        return Some(vec![from_none(typ, types)]);
                    }

                    let c = match &exp.kind {
                        ExpKind::Const(c) if !types.is_pointer(typ) => c,
                        _ => return None,
                    };

                    let i = match c {
                        _ if typ == types.double => return Some(vec![to_f64(*c)]),
                        Const::Double(i) => return Some(vec![from_f64(*i, typ, types)]),
                        Const::Int(i) => *i as u64,
                        Const::Long(i) => *i as u64,
                        Const::UInt(i) => *i as u64,
                        Const::ULong(i) => *i,
                    };

                    match types.kind(typ) {
                        TypeKind::Int => Some(vec![StaticInit::Int(i as i32)]),
                        TypeKind::Long => Some(vec![StaticInit::Long(i as i64)]),
                        TypeKind::UInt => Some(vec![StaticInit::UInt(i as u32)]),
                        TypeKind::ULong => Some(vec![StaticInit::ULong(i)]),
                        _ => unreachable!(),
                    }
                }
                _ => None,
            }
        }

        Some(InitialValue::Initial(inner(init, typ, types)?))
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
    pub fn is_zero(&self) -> bool {
        use StaticInit::*;
        matches!(self, Int(0) | Long(0) | UInt(0) | ULong(0) | ZeroInit(_))
    }
}
