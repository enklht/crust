#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Const(i32),
    KeywordInt,
    KeywordVoid,
    KeywordReturn,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    UnaryMinus,
    UnaryNegation,
    Decrement,
}
