use crate::models::ast::*;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

pub fn parser<'tokens, I>() -> impl Parser<'tokens, I, Program, extra::Err<Rich<'tokens, Token>>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };

    let expr = expr();

    let stmt = recursive(|stmt| {
        let if_ = just(Token::KeywordIf)
            .ignore_then(
                expr.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(stmt.clone().map(Box::new))
            .then(
                just(Token::KeywordElse)
                    .ignore_then(stmt)
                    .map(Box::new)
                    .or_not(),
            )
            .map(|((cond, then), else_)| Stmt::If(cond, then, else_));

        choice((
            expr.clone()
                .then_ignore(just(Token::Semicolon))
                .map(Stmt::Expression),
            just(Token::KeywordReturn)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::Semicolon))
                .map(Stmt::Return),
            if_,
            just(Token::Semicolon).to(Stmt::Null),
        ))
        .labelled("statement")
        .as_context()
    });

    let decl = just(Token::KeywordInt)
        .ignore_then(ident)
        .then(just(Token::Equal).ignore_then(expr).or_not())
        .then_ignore(just(Token::Semicolon))
        .map(|(name, init)| Decl::Decl(name, init))
        .labelled("declaration")
        .as_context();

    let block_item = choice((stmt.map(BlockItem::S), decl.map(BlockItem::D)));

    let func_def = just(Token::KeywordInt)
        .ignore_then(ident)
        .then_ignore(just([Token::LParen, Token::KeywordVoid, Token::RParen]))
        .then(
            block_item
                .repeated()
                .collect()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|(name, body)| FuncDef::Function(name, body));

    let program = func_def.map(Program::FuncDef);

    program.then_ignore(end())
}

fn expr<'tokens, I>() -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        let atom = choice((
            select! {
                Token::Ident(ident) => Expr::Var(ident.clone()),
                Token::Const(i) => Expr::Const(i)
            },
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        atom.pratt((
            postfix(
                14,
                one_of([Token::DoublePlus, Token::DoubleHyphen]),
                |v, op, _| Expr::Unary(convert_unop(op), Box::new(v)),
            ),
            prefix(13, just(Token::DoublePlus), |_, v: Expr, _| {
                Expr::Assignment(
                    Box::new(v.clone()),
                    Box::new(Expr::Binary(
                        BinaryOp::Add,
                        Box::new(v),
                        Box::new(Expr::Const(1)),
                    )),
                )
            }),
            prefix(13, just(Token::DoubleHyphen), |_, v: Expr, _| {
                Expr::Assignment(
                    Box::new(v.clone()),
                    Box::new(Expr::Binary(
                        BinaryOp::Subtract,
                        Box::new(v),
                        Box::new(Expr::Const(1)),
                    )),
                )
            }),
            prefix(
                13,
                one_of([Token::Hyphen, Token::Tilde, Token::Bang]),
                |op, v, _| Expr::Unary(convert_unop(op), Box::new(v)),
            ),
            infix(
                left(12),
                one_of([Token::Star, Token::Slash, Token::Percent]),
                |left, op, right, _| {
                    Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(11),
                one_of([Token::Plus, Token::Hyphen]),
                |left, op, right, _| {
                    Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(10),
                one_of([Token::LShift, Token::RShift]),
                |left, op, right, _| {
                    Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(9),
                one_of([
                    Token::GreaterThan,
                    Token::GreaterOrEqual,
                    Token::LessThan,
                    Token::LessOrEqual,
                ]),
                |left, op, right, _| {
                    Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(8),
                one_of([Token::DoubleEqual, Token::BangEqual]),
                |left, op, right, _| {
                    Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(left(7), just(Token::Ampersand), |left, op, right, _| {
                Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(6), just(Token::Caret), |left, op, right, _| {
                Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(5), just(Token::VerticalBar), |left, op, right, _| {
                Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(4), just(Token::LogicalAnd), |left, op, right, _| {
                Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(3), just(Token::LogicalOr), |left, op, right, _| {
                Expr::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(
                right(2),
                expr.delimited_by(just(Token::Question), just(Token::Colon)),
                |cond, then, else_, _| {
                    Expr::Conditional(Box::new(cond), Box::new(then), Box::new(else_))
                },
            ),
            infix(right(1), just(Token::Equal), |left, _, right, _| {
                Expr::Assignment(Box::new(left), Box::new(right))
            }),
            infix(
                right(1),
                one_of([
                    Token::PlusEqual,
                    Token::HyphenEqual,
                    Token::StarEqual,
                    Token::SlashEqual,
                    Token::PercentEqual,
                    Token::AndEqual,
                    Token::VBarEqual,
                    Token::CaretEqual,
                    Token::LShiftEqual,
                    Token::RShiftEqual,
                ]),
                |left: Expr, op, right, _| {
                    let op = convert_compound(op);
                    Expr::Assignment(
                        Box::new(left.clone()),
                        Box::new(Expr::Binary(op, Box::new(left), Box::new(right))),
                    )
                },
            ),
        ))
    })
    .labelled("expression")
    .as_context()
}

fn convert_binop(token: Token) -> BinaryOp {
    match token {
        Token::Plus => BinaryOp::Add,
        Token::Hyphen => BinaryOp::Subtract,
        Token::Star => BinaryOp::Multiply,
        Token::Slash => BinaryOp::Divide,
        Token::Percent => BinaryOp::Remainder,
        Token::Ampersand => BinaryOp::BitAnd,
        Token::Caret => BinaryOp::BitXor,
        Token::VerticalBar => BinaryOp::BitOr,
        Token::LShift => BinaryOp::LShift,
        Token::RShift => BinaryOp::RShift,
        Token::LogicalAnd => BinaryOp::And,
        Token::LogicalOr => BinaryOp::Or,
        Token::DoubleEqual => BinaryOp::Equal,
        Token::BangEqual => BinaryOp::NotEqual,
        Token::LessThan => BinaryOp::LessThan,
        Token::LessOrEqual => BinaryOp::LessOrEqual,
        Token::GreaterThan => BinaryOp::GreaterThan,
        Token::GreaterOrEqual => BinaryOp::GreaterOrEqual,
        _ => unreachable!(),
    }
}

fn convert_unop(token: Token) -> UnaryOp {
    match token {
        Token::Hyphen => UnaryOp::Negate,
        Token::Tilde => UnaryOp::BitNot,
        Token::Bang => UnaryOp::Not,
        Token::DoublePlus => UnaryOp::PostfixIncrement,
        Token::DoubleHyphen => UnaryOp::PostfixDecrement,
        _ => unreachable!(),
    }
}

fn convert_compound(token: Token) -> BinaryOp {
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
