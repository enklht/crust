use derive_more::Display;
use rustc_hash::FxHashMap;

pub type Symbols = FxHashMap<String, (Type, bool)>;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("Fn(Int{{{_0}}}) -> Int")]
    Fn(usize),
}
