use crate::models::ast::*;
use crate::models::token::Token;
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

type ParseResult<'a, T> = Result<(T, &'a [Token]), ParseError>;

fn expect(expected: Token, tokens: &[Token]) -> Result<&[Token], ParseError> {
    match tokens.first() {
        Some(token) if token == &expected => Ok(&tokens[1..]),
        Some(actual) => Err(ParseError::UnexpectedToken(
            format!("{:?}", expected),
            actual.clone(),
        )),
        None => Err(ParseError::UnexpectedEof(format!("{:?}", expected))),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let (program, rest) = parse_program(&tokens)?;
    if !rest.is_empty() {
        return Err(ParseError::ExpectedEof(rest.first().unwrap().clone()));
    }

    Ok(program)
}

fn parse_program(tokens: &[Token]) -> ParseResult<'_, Program> {
    let (function, rest) = parse_function(tokens)?;

    Ok((Program::Function(function), rest))
}

fn parse_function(tokens: &[Token]) -> ParseResult<'_, Function> {
    let rest = expect(Token::KeywordInt, tokens)?;
    let (name, rest) = parse_ident(rest)?;
    let rest = expect(Token::LParen, rest)?;
    let rest = expect(Token::KeywordVoid, rest)?;
    let rest = expect(Token::RParen, rest)?;
    let rest = expect(Token::LBrace, rest)?;
    let (body, rest) = parse_stmt(rest)?;
    let rest = expect(Token::RBrace, rest)?;

    Ok((Function::Function { name, body }, rest))
}

fn parse_stmt(tokens: &[Token]) -> ParseResult<'_, Stmt> {
    let rest = expect(Token::KeywordReturn, tokens)?;
    let (val, rest) = parse_expr(rest)?;
    let rest = expect(Token::Semicolon, rest)?;

    Ok((Stmt::Return(val), rest))
}

fn parse_expr(tokens: &[Token]) -> ParseResult<'_, Expr> {
    if let Ok((int, rest)) = parse_int(tokens) {
        Ok((Expr::Int(int), rest))
    } else {
        match tokens.first() {
            Some(Token::Tilde) | Some(Token::Minus) => {
                let (op, rest) = parse_unop(tokens)?;
                let (expr, rest) = parse_expr(rest)?;
                Ok((Expr::Int(Int::Unary(op, Box::new(expr))), rest))
            }
            Some(Token::LParen) => {
                let (expr, rest) = parse_expr(&tokens[1..])?;
                let rest = expect(Token::RParen, rest)?;
                Ok((expr, rest))
            }
            Some(actual) => Err(ParseError::UnexpectedToken(
                "expr".to_string(),
                actual.clone(),
            )),
            None => Err(ParseError::UnexpectedEof("expr".to_string())),
        }
    }
}

fn parse_unop(tokens: &[Token]) -> ParseResult<'_, UnaryOperator> {
    match tokens.first() {
        Some(Token::Tilde) => Ok((UnaryOperator::Complement, &tokens[1..])),
        Some(Token::Minus) => Ok((UnaryOperator::Negate, &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "unary operator".to_string(),
            actual.clone(),
        )),
        None => Err(ParseError::UnexpectedEof("unary operator".to_string())),
    }
}

fn parse_ident(tokens: &[Token]) -> ParseResult<'_, Ident> {
    match tokens.first() {
        Some(Token::Ident(name)) => Ok((Ident::Ident(name.clone()), &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "ident".to_string(),
            actual.clone(),
        )),
        _ => Err(ParseError::UnexpectedEof("ident".to_string())),
    }
}

fn parse_int(tokens: &[Token]) -> ParseResult<'_, Int> {
    match tokens.first() {
        Some(Token::Const(i)) => Ok((Int::Const(*i), &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "const".to_string(),
            actual.clone(),
        )),
        _ => Err(ParseError::UnexpectedEof("const".to_string())),
    }
}
