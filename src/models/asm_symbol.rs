use super::assembly::AsmType;
use rustc_hash::FxHashMap;

type Ident = &'static str;
pub type AsmSymbols = FxHashMap<Ident, AsmSymbol>;

pub enum AsmSymbol {
    ObjEntry(AsmType, bool, bool),
    FunEntry(bool),
}
