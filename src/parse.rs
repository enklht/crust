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

    Ok((Function::Function(name, body), rest))
}

fn parse_stmt(tokens: &[Token]) -> ParseResult<'_, Stmt> {
    let rest = expect(Token::KeywordReturn, tokens)?;
    let (val, rest) = parse_expr(rest, 0)?;
    let rest = expect(Token::Semicolon, rest)?;

    Ok((Stmt::Return(val), rest))
}

fn precedence(token: &Token) -> Option<u8> {
    match token {
        Token::Star | Token::Slash | Token::Percent => Some(50),
        Token::Plus | Token::Minus => Some(45),
        _ => None,
    }
}

fn parse_expr(tokens: &[Token], min_prec: u8) -> ParseResult<'_, Expr> {
    let (mut left, mut rest_outer) = parse_factor(tokens)?;

    while let Some(token) = rest_outer.first()
        && let Some(prec) = precedence(token)
        && prec > min_prec
    {
        let (operator, rest) = parse_binop(rest_outer)?;
        let (right, rest) = parse_expr(rest, precedence(token).unwrap() + 1)?;
        rest_outer = rest;
        left = Expr::Binary(operator, Box::new(left), Box::new(right));
    }

    Ok((left, rest_outer))
}

fn parse_factor(tokens: &[Token]) -> ParseResult<'_, Expr> {
    match tokens.first() {
        Some(Token::Const(i)) => Ok((Expr::Const(*i), &tokens[1..])),
        Some(Token::Tilde) | Some(Token::Minus) => {
            let (op, rest) = parse_unop(tokens)?;
            let (expr, rest) = parse_factor(rest)?;
            Ok((Expr::Unary(op, Box::new(expr)), rest))
        }
        Some(Token::LParen) => {
            let (expr, rest) = parse_expr(&tokens[1..], 0)?;
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

fn parse_unop(tokens: &[Token]) -> ParseResult<'_, UnaryOp> {
    match tokens.first() {
        Some(Token::Tilde) => Ok((UnaryOp::Complement, &tokens[1..])),
        Some(Token::Minus) => Ok((UnaryOp::Negate, &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "unary operator".to_string(),
            actual.clone(),
        )),
        None => Err(ParseError::UnexpectedEof("unary operator".to_string())),
    }
}

fn parse_binop(tokens: &[Token]) -> ParseResult<'_, BinaryOp> {
    match tokens.first() {
        Some(Token::Plus) => Ok((BinaryOp::Add, &tokens[1..])),
        Some(Token::Minus) => Ok((BinaryOp::Subtract, &tokens[1..])),
        Some(Token::Star) => Ok((BinaryOp::Multiply, &tokens[1..])),
        Some(Token::Slash) => Ok((BinaryOp::Divide, &tokens[1..])),
        Some(Token::Percent) => Ok((BinaryOp::Remainder, &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "unary operator".to_string(),
            actual.clone(),
        )),
        None => Err(ParseError::UnexpectedEof("unary operator".to_string())),
    }
}

fn parse_ident(tokens: &[Token]) -> ParseResult<'_, String> {
    match tokens.first() {
        Some(Token::Ident(name)) => Ok((name.clone(), &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "ident".to_string(),
            actual.clone(),
        )),
        _ => Err(ParseError::UnexpectedEof("ident".to_string())),
    }
}
