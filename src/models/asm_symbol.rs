use super::assembly::AssemblyType;
use super::symbol::{Symbol, Symbols};
use rustc_hash::FxHashMap;

pub type AsmSymbols = FxHashMap<String, AsmSymbol>;

pub enum AsmSymbol {
    ObjEntry(AssemblyType, bool),
    FunEntry(bool),
}

pub fn from_symbols(symbols: Symbols) -> AsmSymbols {
    symbols
        .iter()
        .map(|(name, symbol)| match symbol {
            Symbol::Func { defined, .. } => (name.clone(), AsmSymbol::FunEntry(*defined)),
            Symbol::StaticVar { typ, .. } => {
                (name.clone(), AsmSymbol::ObjEntry(typ.to_asm_type(), true))
            }
            Symbol::LocalVar { typ } => {
                (name.clone(), AsmSymbol::ObjEntry(typ.to_asm_type(), false))
            }
        })
        .collect()
}
