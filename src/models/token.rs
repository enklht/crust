use logos::Logos;

#[derive(Debug, PartialEq, Clone, Logos)]
#[logos(skip r"\s+|//.*\n|/\*[^(\*/)]*")]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().unwrap())]
    Const(i32),
    #[token("int")]
    KeywordInt,
    #[token("void")]
    KeywordVoid,
    #[token("return")]
    KeywordReturn,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    #[token("-")]
    Minus,
    #[token("--")]
    Decrement,
    #[token("~")]
    Tilde,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    Ampersand,
    #[token("|")]
    VerticalBar,
    #[token("^")]
    Caret,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,
    #[token("!")]
    Bang,
    #[token("&&")]
    LogicalAnd,
    #[token("||")]
    LogicalOr,
    #[token("==")]
    DoubleEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessOrEqual,
    #[token(">=")]
    GreaterOrEqual,
}
