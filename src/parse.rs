use crate::models::ast::*;
use crate::models::token::Token;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected {0}, found {1:?}")]
    UnexpectedToken(String, String),
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
        Token::Equal
        | Token::PlusEqual
        | Token::HyphenEqual
        | Token::StarEqual
        | Token::SlashEqual
        | Token::PercentEqual
        | Token::AndEqual
        | Token::VBarEqual
        | Token::CaretEqual
        | Token::LShiftEqual
        | Token::RShiftEqual => Some(14),
        _ => None,
    }
}

fn expect(expected: Token, tokens: &[Token]) -> Result<&[Token], ParseError> {
    match tokens.first() {
        Some(token) if token == &expected => Ok(&tokens[1..]),
        Some(actual) => Err(ParseError::UnexpectedToken(
            format!("{:?}", expected),
            format!("{:?}", actual),
        )),
        None => Err(ParseError::UnexpectedToken(
            format!("{:?}", expected),
            "EOF".to_string(),
        )),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let (program, rest) = parse_program(&tokens)?;
    if !rest.is_empty() {
        return Err(ParseError::UnexpectedToken(
            "EOF".to_string(),
            format!("{:?}", rest[0]),
        ));
    }

    Ok(program)
}

fn parse_program(tokens: &[Token]) -> ParseResult<'_, Program> {
    let (function, rest) = parse_function(tokens)?;

    Ok((Program::FuncDef(function), rest))
}

fn parse_function(tokens: &[Token]) -> ParseResult<'_, FuncDef> {
    let name: String;
    let mut rest = expect(Token::KeywordInt, tokens)?;
    (name, rest) = parse_ident(rest)?;
    rest = expect(Token::LParen, rest)?;
    rest = expect(Token::KeywordVoid, rest)?;
    rest = expect(Token::RParen, rest)?;
    rest = expect(Token::LBrace, rest)?;

    let mut body = Vec::new();
    let mut next_item: BlockItem;
    while rest.first() != Some(&Token::RBrace) {
        (next_item, rest) = parse_block_item(rest)?;
        body.push(next_item);
    }
    let rest = expect(Token::RBrace, rest)?;

    Ok((FuncDef::Function(name, body), rest))
}

fn parse_block_item(tokens: &[Token]) -> ParseResult<'_, BlockItem> {
    if let Ok((decl, rest)) = parse_decl(tokens) {
        Ok((BlockItem::D(decl), rest))
    } else if let Ok((stmt, rest)) = parse_stmt(tokens) {
        Ok((BlockItem::S(stmt), rest))
    } else {
        Err(ParseError::UnexpectedToken(
            "<decl> or <stmt>".to_string(),
            format!("{:?}", tokens),
        ))
    }
}

fn parse_stmt(tokens: &[Token]) -> ParseResult<'_, Stmt> {
    match tokens.first() {
        Some(Token::KeywordReturn) => {
            let (val, rest) = parse_expr(&tokens[1..], u8::MAX)?;
            let rest = expect(Token::Semicolon, rest)?;
            Ok((Stmt::Return(val), rest))
        }
        Some(Token::Semicolon) => Ok((Stmt::Null, &tokens[1..])),
        _ => {
            let (expr, rest) = parse_expr(tokens, u8::MAX)?;
            let rest = expect(Token::Semicolon, rest)?;
            Ok((Stmt::Expression(expr), rest))
        }
    }
}

fn parse_decl(tokens: &[Token]) -> ParseResult<'_, Decl> {
    let rest = expect(Token::KeywordInt, tokens)?;
    let (name, rest) = parse_ident(rest)?;
    match rest.first() {
        Some(Token::Equal) => {
            let (expr, rest) = parse_expr(&rest[1..], u8::MAX)?;
            let rest = expect(Token::Semicolon, rest)?;
            Ok((Decl::Decl(name, Some(expr)), rest))
        }
        Some(Token::Semicolon) => Ok((Decl::Decl(name, None), &rest[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "=".to_string(),
            format!("{:?}", actual),
        )),
        None => Err(ParseError::UnexpectedToken(
            "=".to_string(),
            "EOF".to_string(),
        )),
    }
}

fn parse_expr(tokens: &[Token], max_prec: u8) -> ParseResult<'_, Expr> {
    let (mut left, mut rest) = parse_factor(tokens)?;

    while let Some(token) = rest.first()
        && let Some(prec) = precedence(token)
        && prec <= max_prec
    {
        match token {
            Token::Equal => {
                let (right, rest_inner) = parse_expr(&rest[1..], prec)?;
                rest = rest_inner;
                left = Expr::Assignment(Box::new(left), Box::new(right));
            }
            Token::PlusEqual
            | Token::HyphenEqual
            | Token::StarEqual
            | Token::SlashEqual
            | Token::PercentEqual
            | Token::AndEqual
            | Token::VBarEqual
            | Token::CaretEqual
            | Token::LShiftEqual
            | Token::RShiftEqual => {
                let operator = convert_compound(token);
                let (right, rest_inner) = parse_expr(&rest[1..], prec)?;
                rest = rest_inner;
                left = Expr::Assignment(
                    Box::new(left.clone()),
                    Box::new(Expr::Binary(operator, Box::new(left), Box::new(right))),
                );
            }
            _ => {
                let (operator, rest_inner) = parse_binop(rest)?;
                let (right, rest_inner) = parse_expr(rest_inner, prec - 1)?;
                rest = rest_inner;
                left = Expr::Binary(operator, Box::new(left), Box::new(right));
            }
        }
    }

    Ok((left, rest))
}

fn parse_factor(tokens: &[Token]) -> ParseResult<'_, Expr> {
    if let Ok((op, rest)) = parse_unop(tokens) {
        let (expr, rest) = parse_factor(rest)?;
        let expr = match op {
            UnaryOp::PrefixIncrement => Expr::Assignment(
                Box::new(expr.clone()),
                Box::new(Expr::Binary(
                    BinaryOp::Add,
                    Box::new(expr),
                    Box::new(Expr::Const(1)),
                )),
            ),
            UnaryOp::PrefixDecrement => Expr::Assignment(
                Box::new(expr.clone()),
                Box::new(Expr::Binary(
                    BinaryOp::Subtract,
                    Box::new(expr),
                    Box::new(Expr::Const(1)),
                )),
            ),
            op => Expr::Unary(op, Box::new(expr)),
        };

        Ok((expr, rest))
    } else {
        parse_primary(tokens)
    }
}

fn parse_primary(tokens: &[Token]) -> ParseResult<'_, Expr> {
    let (mut primary, mut rest) = parse_atom(tokens)?;

    loop {
        match rest.first() {
            Some(Token::DoublePlus) => {
                primary = Expr::Unary(UnaryOp::PostfixIncrement, Box::new(primary));
                rest = &rest[1..];
            }
            Some(Token::DoubleHyphen) => {
                primary = Expr::Unary(UnaryOp::PostfixDecrement, Box::new(primary));
                rest = &rest[1..];
            }
            _ => break,
        }
    }

    Ok((primary, rest))
}

fn parse_atom(tokens: &[Token]) -> ParseResult<'_, Expr> {
    match tokens.first() {
        Some(Token::Const(i)) => Ok((Expr::Const(*i), &tokens[1..])),
        Some(Token::Ident(ident)) => Ok((Expr::Var(ident.clone()), &tokens[1..])),
        Some(Token::LParen) => {
            let (expr, rest) = parse_expr(&tokens[1..], u8::MAX)?;
            let rest = expect(Token::RParen, rest)?;
            Ok((expr, rest))
        }
        Some(actual) => Err(ParseError::UnexpectedToken(
            "<expr>".to_string(),
            format!("{:?}", actual),
        )),
        None => Err(ParseError::UnexpectedToken(
            "<expr>".to_string(),
            "EOF".to_string(),
        )),
    }
}

fn parse_unop(tokens: &[Token]) -> ParseResult<'_, UnaryOp> {
    let unop = match tokens.first() {
        Some(Token::Tilde) => UnaryOp::Complement,
        Some(Token::Hyphen) => UnaryOp::Negate,
        Some(Token::Bang) => UnaryOp::Not,
        Some(Token::DoublePlus) => UnaryOp::PrefixIncrement,
        Some(Token::DoubleHyphen) => UnaryOp::PrefixDecrement,
        Some(actual) => {
            return Err(ParseError::UnexpectedToken(
                "<unary operator>".to_string(),
                format!("{:?}", actual),
            ));
        }
        None => {
            return Err(ParseError::UnexpectedToken(
                "<unary operator>".to_string(),
                "EOF".to_string(),
            ));
        }
    };

    Ok((unop, &tokens[1..]))
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
                "<binary operator>".to_string(),
                format!("{:?}", actual),
            ));
        }
        None => {
            return Err(ParseError::UnexpectedToken(
                "<binary operator>".to_string(),
                "EOF".to_string(),
            ));
        }
    };

    Ok((operator, &tokens[1..]))
}

fn convert_compound(token: &Token) -> BinaryOp {
    match token {
        Token::PlusEqual => BinaryOp::Add,
        Token::HyphenEqual => BinaryOp::Subtract,
        Token::StarEqual => BinaryOp::Multiply,
        Token::SlashEqual => BinaryOp::Divide,
        Token::PercentEqual => BinaryOp::Remainder,
        Token::AndEqual => BinaryOp::BitAnd,
        Token::VBarEqual => BinaryOp::BitOr,
        Token::CaretEqual => BinaryOp::BitXor,
        Token::LShiftEqual => BinaryOp::LShift,
        Token::RShiftEqual => BinaryOp::RShift,
        _ => unreachable!(),
    }
}

fn parse_ident(tokens: &[Token]) -> ParseResult<'_, String> {
    match tokens.first() {
        Some(Token::Ident(name)) => Ok((name.clone(), &tokens[1..])),
        Some(actual) => Err(ParseError::UnexpectedToken(
            "ident".to_string(),
            format!("{:?}", actual),
        )),
        _ => Err(ParseError::UnexpectedToken(
            "ident".to_string(),
            "EOF".to_string(),
        )),
    }
}
