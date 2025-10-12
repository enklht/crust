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

fn precedence(token: &Token) -> Option<u8> {
    match token {
        Token::Star | Token::Slash | Token::Percent => Some(3),
        Token::Plus | Token::Hyphen => Some(4),
        Token::LShift | Token::RShift => Some(5),
        Token::LessThan | Token::LessOrEqual | Token::GreaterThan | Token::GreaterOrEqual => {
            Some(6)
        }
        Token::DoubleEqual | Token::BangEqual => Some(7),
        Token::Ampersand => Some(8),
        Token::Caret => Some(9),
        Token::VerticalBar => Some(10),
        Token::LogicalAnd => Some(11),
        Token::LogicalOr => Some(12),
        _ => None,
    }
}

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
        Some(Token::Tilde) | Some(Token::Hyphen) | Some(Token::Bang) => {
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
        Some(Token::Hyphen) => Ok((UnaryOp::Negate, &tokens[1..])),
        Some(Token::Bang) => Ok((UnaryOp::Not, &tokens[1..])),
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
        Some(Token::Hyphen) => BinaryOp::Subtract,
        Some(Token::Star) => BinaryOp::Multiply,
        Some(Token::Slash) => BinaryOp::Divide,
        Some(Token::Percent) => BinaryOp::Remainder,
        Some(Token::Ampersand) => BinaryOp::BitAnd,
        Some(Token::Caret) => BinaryOp::BitXor,
        Some(Token::VerticalBar) => BinaryOp::BitOr,
        Some(Token::LShift) => BinaryOp::LShift,
        Some(Token::RShift) => BinaryOp::RShift,
        Some(Token::LogicalAnd) => BinaryOp::And,
        Some(Token::LogicalOr) => BinaryOp::Or,
        Some(Token::DoubleEqual) => BinaryOp::Equal,
        Some(Token::BangEqual) => BinaryOp::NotEqual,
        Some(Token::LessThan) => BinaryOp::LessThan,
        Some(Token::LessOrEqual) => BinaryOp::LessOrEqual,
        Some(Token::GreaterThan) => BinaryOp::GreaterThan,
        Some(Token::GreaterOrEqual) => BinaryOp::GreaterOrEqual,
        Some(actual) => {
            return Err(ParseError::UnexpectedToken(
                "binary operator".to_string(),
                actual.clone(),
            ));
        }
        None => return Err(ParseError::UnexpectedEof("binary operator".to_string())),
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
