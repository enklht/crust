use super::assembly::AsmType;
use crate::utils::Ident;
use rustc_hash::FxHashMap;

pub type AsmSymbols = FxHashMap<Ident, AsmSymbol>;

pub enum AsmSymbol {
    ObjEntry(AsmType, bool, bool),
    FunEntry(bool),
}
