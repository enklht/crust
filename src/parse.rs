use crate::models::ast::*;
use crate::models::token::Token;
use std::iter::Peekable;
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

type Lexer<'a> = Peekable<logos::Lexer<'a, Token>>;
type ParseResult<T> = Result<T, ParseError>;

fn precedence(token: &Token) -> Option<u8> {
    match token {
        Token::Star | Token::Slash | Token::Percent => Some(3),
        Token::Plus | Token::Minus => Some(4),
        Token::LShift | Token::RShift => Some(5),
        Token::Ampersand => Some(8),
        Token::Caret => Some(9),
        Token::VerticalBar => Some(10),
        _ => None,
    }
}

fn expect(expected: Token, lexer: &mut Lexer) -> ParseResult<()> {
    match lexer.peek() {
        Some(token) if token == Ok(&expected) => Ok(()),
        Some(actual) => Err(ParseError::UnexpectedToken(
            format!("{:?}", expected),
            actual.unwrap().clone(),
        )),
        None => Err(ParseError::UnexpectedEof(format!("{:?}", expected))),
    }
}

pub fn parse(lexer: &mut Lexer) -> Result<Program, ParseError> {
    let program = parse_program(lexer)?;

    match lexer.next() {
        Some(token) => Err(ParseError::ExpectedEof(token.unwrap())),
        None => Ok(program),
    }
}

fn parse_program(lexer: &mut Lexer) -> Result<Program, ParseError> {
    let function = parse_function(lexer)?;

    Ok(Program::Function(function))
}

fn parse_function(tokens: &mut Lexer) -> ParseResult<Function> {
    todo!();
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
    let (val, rest) = parse_expr(rest, 255)?;
    let rest = expect(Token::Semicolon, rest)?;

    Ok((Stmt::Return(val), rest))
}

fn parse_expr(tokens: &[Token], max_prec: u8) -> ParseResult<'_, Expr> {
    let (mut left, mut rest) = parse_factor(tokens)?;

    while let Some(token) = rest.first()
        && let Some(prec) = precedence(token)
        && prec <= max_prec
    {
        let (operator, rest_inner) = parse_binop(rest)?;
        let (right, rest_inner) = parse_expr(rest_inner, prec - 1)?;
        rest = rest_inner;
        left = Expr::Binary(operator, Box::new(left), Box::new(right));
    }

    Ok((left, rest))
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
            let (expr, rest) = parse_expr(&tokens[1..], 255)?;
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
    let operator = match tokens.first() {
        Some(Token::Plus) => BinaryOp::Add,
        Some(Token::Minus) => BinaryOp::Subtract,
        Some(Token::Star) => BinaryOp::Multiply,
        Some(Token::Slash) => BinaryOp::Divide,
        Some(Token::Percent) => BinaryOp::Remainder,
        Some(Token::Ampersand) => BinaryOp::BitAnd,
        Some(Token::Caret) => BinaryOp::BitXor,
        Some(Token::VerticalBar) => BinaryOp::BitOr,
        Some(Token::LShift) => BinaryOp::LShift,
        Some(Token::RShift) => BinaryOp::RShift,
        Some(actual) => {
            return Err(ParseError::UnexpectedToken(
                "unary operator".to_string(),
                actual.clone(),
            ));
        }
        None => return Err(ParseError::UnexpectedEof("unary operator".to_string())),
    };

    Ok((operator, &tokens[1..]))
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
