use super::assembly::AsmType;
use super::symbol::Symbol;
use super::symbol::Symbols;
use rustc_hash::FxHashMap;

pub type AsmSymbols = FxHashMap<String, AsmSymbol>;

pub enum AsmSymbol {
    ObjEntry(AsmType, bool),
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
