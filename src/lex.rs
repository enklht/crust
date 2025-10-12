use crate::models::token::Token;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unknown token: {0}")]
    UnknownToken(String),
}

type Converter = fn(&str) -> Token;

fn convert_ident(input: &str) -> Token {
    match input {
        "int" => Token::KeywordInt,
        "return" => Token::KeywordReturn,
        "void" => Token::KeywordVoid,
        _ => Token::Ident(input.to_string()),
    }
}

fn convert_const(input: &str) -> Token {
    Token::Const(input.parse().unwrap())
}

pub fn lex(mut input: &str) -> Result<Vec<Token>, LexError> {
    let rules: Vec<(Regex, Converter)> = vec![
        (Regex::new(r"^[a-zA-Z_]\w*\b").unwrap(), convert_ident),
        (Regex::new(r"^[0-9]+\b").unwrap(), convert_const),
        (Regex::new(r"^\(").unwrap(), |_| Token::LParen),
        (Regex::new(r"^\)").unwrap(), |_| Token::RParen),
        (Regex::new(r"^\{").unwrap(), |_| Token::LBrace),
        (Regex::new(r"^\}").unwrap(), |_| Token::RBrace),
        (Regex::new(r"^;").unwrap(), |_| Token::Semicolon),
        (Regex::new(r"^-").unwrap(), |_| Token::Minus),
        (Regex::new(r"^~").unwrap(), |_| Token::Tilde),
        (Regex::new(r"^--").unwrap(), |_| Token::Decrement),
    ];
    let whitespace = Regex::new(r"^\s+|^//.*?\n|^/\*.*?\*/").unwrap();

    let mut tokens: Vec<Token> = Vec::new();

    while !input.is_empty() {
        if let Some(m) = whitespace.find(input) {
            input = &input[m.end()..];
            continue;
        }

        let mut best = None;

        for (re, f) in &rules {
            if let Some(mat) = re.find(input) {
                let len = mat.end();
                if best.as_ref().is_none_or(|(_, l)| len > *l) {
                    best = Some((f(mat.as_str()), len));
                }
            }
        }

        let Some((token, len)) = best else {
            return Err(LexError::UnknownToken(input.to_string()));
        };
        tokens.push(token);
        input = &input[len..]
    }

    Ok(tokens)
}
