use super::assembly::AsmType;
use super::symbol::Symbol;
use super::symbol::Symbols;
use rustc_hash::FxHashMap;

type Ident = &'static str;
pub type AsmSymbols = FxHashMap<Ident, AsmSymbol>;

pub enum AsmSymbol {
    ObjEntry(AsmType, bool),
    FunEntry(bool),
}

pub fn from_symbols(symbols: Symbols) -> AsmSymbols {
    symbols
        .into_iter()
        .map(|(name, symbol)| match symbol {
            Symbol::Func { defined, .. } => (name, AsmSymbol::FunEntry(defined)),
            Symbol::StaticVar { typ, .. } => (name, AsmSymbol::ObjEntry(typ.to_asm_type(), true)),
            Symbol::LocalVar { typ } => (name, AsmSymbol::ObjEntry(typ.to_asm_type(), false)),
        })
        .collect()
}
