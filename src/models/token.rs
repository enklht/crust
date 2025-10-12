#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Const(i64),
    KeywordInt,
    KeywordVoid,
    KeywordReturn,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Minus,
    Tilde,
    Decrement,
}
