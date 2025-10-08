#[derive(Debug, PartialEq)]
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
}
