use crate::model::Token;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("syntax error")]
    SyntaxError,
}

pub enum Program {
    Function(Function),
}

pub struct Function {
    name: String,
    body: Stmt,
}

pub enum Stmt {
    Return(Expr),
}

pub enum Expr {
    Int(i32),
}

fn expect(expected: Token, tokens: &mut Vec<Token>) -> Result<Token, ParseError> {
    if Some(&expected) == tokens.last() {
        tokens.pop().ok_or(ParseError::SyntaxError)
    } else {
        Err(ParseError::SyntaxError)
    }
}

pub fn parse(mut tokens: Vec<Token>) -> Result<Program, ParseError> {
    tokens.reverse();
    let program = parse_program(&mut tokens)?;
    if !tokens.is_empty() {
        return Err(ParseError::SyntaxError);
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
        _ => return Err(ParseError::SyntaxError),
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
        Some(Token::Const(n)) => Ok(Expr::Int(n)),
        _ => Err(ParseError::SyntaxError),
    }
}
