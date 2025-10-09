use crate::token::Token;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected {0}, found {1:?}")]
    UnexpectedToken(String, Token),
    #[error("expected {0}, found <eof>")]
    UnexpectedEof(String),
    #[error("expected <eof>, found {0:?}")]
    ExpectedEof(Token),
}

#[derive(Debug)]
pub enum Program {
    Function(Function),
}

#[derive(Debug)]
pub struct Function {
    name: String,
    body: Stmt,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Const(i32),
}

fn expect(expected: Token, tokens: &mut Vec<Token>) -> Result<Token, ParseError> {
    match tokens.last() {
        Some(token) if token == &expected => Ok(tokens.pop().unwrap()),
        Some(actual) => Err(ParseError::UnexpectedToken(
            format!("{:?}", expected),
            actual.clone(),
        )),
        None => Err(ParseError::UnexpectedEof(format!("{:?}", expected))),
    }
}

pub fn parse(mut tokens: Vec<Token>) -> Result<Program, ParseError> {
    tokens.reverse();
    let program = parse_program(&mut tokens)?;
    if !tokens.is_empty() {
        return Err(ParseError::ExpectedEof(tokens.pop().unwrap()));
    }

    Ok(program)
}

fn parse_program(tokens: &mut Vec<Token>) -> Result<Program, ParseError> {
    let function = parse_function(tokens)?;

    Ok(Program::Function(function))
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<Function, ParseError> {
    expect(Token::KeywordInt, tokens)?;
    let name = match tokens.pop() {
        Some(Token::Ident(name)) => name,
        Some(actual) => return Err(ParseError::UnexpectedToken("ident".to_string(), actual)),
        None => return Err(ParseError::UnexpectedEof("ident".to_string())),
    };
    expect(Token::LParen, tokens)?;
    expect(Token::KeywordVoid, tokens)?;
    expect(Token::RParen, tokens)?;
    expect(Token::LBrace, tokens)?;
    let body = parse_stmt(tokens)?;
    expect(Token::RBrace, tokens)?;

    Ok(Function { name, body })
}

fn parse_stmt(tokens: &mut Vec<Token>) -> Result<Stmt, ParseError> {
    expect(Token::KeywordReturn, tokens)?;
    let return_val = parse_expr(tokens)?;
    expect(Token::Semicolon, tokens)?;

    Ok(Stmt::Return(return_val))
}

fn parse_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParseError> {
    match tokens.pop() {
        Some(Token::Const(n)) => Ok(Expr::Const(n)),
        Some(actual) => Err(ParseError::UnexpectedToken("const".to_string(), actual)),
        None => Err(ParseError::UnexpectedEof("const".to_string())),
    }
}
