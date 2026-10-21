use super::ParserError;
use super::array_suffix;
use super::type_specifiers;
use crate::models::ast::*;
use crate::models::token::Token;
use crate::models::typ::Type;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

fn binary(op: BinaryOp, left: Exp, right: Exp) -> Exp {
    Exp::new(ExpKind::Binary(op, Box::new(left), Box::new(right)))
}

fn update(op: BinaryOp, left: Exp, right: Exp, postfix: bool) -> Exp {
    Exp::new(ExpKind::Update(
        op,
        Box::new(left),
        Box::new(right),
        postfix,
    ))
}

pub fn exp<'tokens, I>() -> impl Parser<'tokens, I, Exp, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|exp| {
        let argument_list = exp.clone().separated_by(just(Token::Comma)).collect();

        let func_call = select! { Token::Ident(ident) => ident }
            .then(argument_list.delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(name, args)| Exp::new(ExpKind::FuncCall(name, args)));

        let atom = choice((
            func_call,
            select! {
                Token::Ident(ident) => Exp::new(ExpKind::Var(ident)),
                Token::ConstInt(i) => Exp::new(ExpKind::Const(Const::Int(i))),
                Token::ConstLong(i) => Exp::new(ExpKind::Const(Const::Long(i))),
                Token::ConstUInt(i) => Exp::new(ExpKind::Const(Const::UInt(i))),
                Token::ConstULong(i) => Exp::new(ExpKind::Const(Const::ULong(i))),
                Token::ConstDouble(i) => Exp::new(ExpKind::Const(Const::Double(i))),
            },
            exp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        atom.pratt((
            postfix(14, just(Token::DoublePlus), |v: Exp, _, _| {
                update(
                    BinaryOp::Add,
                    v,
                    Exp::new(ExpKind::Const(Const::Int(1))),
                    true,
                )
            }),
            postfix(14, just(Token::DoubleHyphen), |v: Exp, _, _| {
                update(
                    BinaryOp::Subtract,
                    v,
                    Exp::new(ExpKind::Const(Const::Int(1))),
                    true,
                )
            }),
            postfix(
                14,
                exp.clone()
                    .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                |v: Exp, i: Exp, _| Exp::new(ExpKind::Subscript(Box::new(v), Box::new(i))),
            ),
            prefix(13, just(Token::DoublePlus), |_, v: Exp, _| {
                update(
                    BinaryOp::Add,
                    v,
                    Exp::new(ExpKind::Const(Const::Int(1))),
                    false,
                )
            }),
            prefix(13, just(Token::DoubleHyphen), |_, v: Exp, _| {
                update(
                    BinaryOp::Subtract,
                    v,
                    Exp::new(ExpKind::Const(Const::Int(1))),
                    false,
                )
            }),
            prefix(
                13,
                one_of([Token::Hyphen, Token::Tilde, Token::Bang]),
                |op, v, _| Exp::new(ExpKind::Unary(convert_unop(op), Box::new(v))),
            ),
            prefix(13, cast_op(), |typ, v, _| {
                Exp::new(ExpKind::Cast(typ, Box::new(v)))
            }),
            prefix(13, just(Token::Ampersand), |_, v, _| {
                Exp::new(ExpKind::AddressOf(Box::new(v)))
            }),
            prefix(13, just(Token::Star), |_, v, _| {
                Exp::new(ExpKind::Dereference(Box::new(v)))
            }),
            infix(
                left(12),
                one_of([Token::Star, Token::Slash, Token::Percent]),
                |left, op, right, _| binary(convert_binop(op), left, right),
            ),
            infix(
                left(11),
                one_of([Token::Plus, Token::Hyphen]),
                |left, op, right, _| binary(convert_binop(op), left, right),
            ),
            infix(
                left(10),
                one_of([Token::LShift, Token::RShift]),
                |left, op, right, _| binary(convert_binop(op), left, right),
            ),
            infix(
                left(9),
                one_of([
                    Token::GreaterThan,
                    Token::GreaterOrEqual,
                    Token::LessThan,
                    Token::LessOrEqual,
                ]),
                |left, op, right, _| binary(convert_binop(op), left, right),
            ),
            infix(
                left(8),
                one_of([Token::DoubleEqual, Token::BangEqual]),
                |left, op, right, _| binary(convert_binop(op), left, right),
            ),
            infix(left(7), just(Token::Ampersand), |left, _, right, _| {
                binary(BinaryOp::BitAnd, left, right)
            }),
            infix(left(6), just(Token::Caret), |left, _, right, _| {
                binary(BinaryOp::BitXor, left, right)
            }),
            infix(left(5), just(Token::VerticalBar), |left, _, right, _| {
                binary(BinaryOp::BitOr, left, right)
            }),
            infix(left(4), just(Token::LogicalAnd), |left, _, right, _| {
                binary(BinaryOp::LogicalAnd, left, right)
            }),
            infix(left(3), just(Token::LogicalOr), |left, _, right, _| {
                binary(BinaryOp::LogicalOr, left, right)
            }),
            infix(
                right(2),
                exp.delimited_by(just(Token::Question), just(Token::Colon)),
                |cond, then, else_, _| {
                    Exp::new(ExpKind::Conditional(
                        Box::new(cond),
                        Box::new(then),
                        Box::new(else_),
                    ))
                },
            ),
            infix(right(1), just(Token::Equal), |left, _, right, _| {
                Exp::new(ExpKind::Assignment(Box::new(left), Box::new(right)))
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
                |left, op, right, _| update(convert_compound(op), left, right, false),
            ),
        ))
    })
    .labelled("expession")
    .as_context()
}

fn cast_op<'tokens, I>() -> impl Parser<'tokens, I, Type, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    type_specifiers()
        .then(abs_declarator())
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .try_map(|(base_type, abs_declarator), span| {
            process_abs_declarator(abs_declarator, base_type)
                .ok_or(Rich::custom(span, "invalid abstract declarator"))
        })
}

#[derive(Clone)]
enum AbsDeclarator {
    Base,
    Pointer(Box<AbsDeclarator>),
    Array(Box<AbsDeclarator>, isize),
}

fn abs_declarator<'tokens, I>()
-> impl Parser<'tokens, I, AbsDeclarator, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|abs_declarator| {
        let direct_abs_declarator = choice((
            abs_declarator
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
            empty().to(AbsDeclarator::Base),
        ))
        .foldl(array_suffix().repeated(), |base, size| {
            AbsDeclarator::Array(Box::new(base), size)
        });

        choice((
            just(Token::Star)
                .ignore_then(abs_declarator)
                .map(|abs_declarator| AbsDeclarator::Pointer(Box::new(abs_declarator))),
            direct_abs_declarator,
        ))
    })
}

fn process_abs_declarator(abs_declarator: AbsDeclarator, base_typ: Type) -> Option<Type> {
    use AbsDeclarator::*;
    match abs_declarator {
        Base => Some(base_typ),
        Pointer(inner) => process_abs_declarator(*inner, Type::Pointer(Box::new(base_typ))),
        Array(inner, size) => process_abs_declarator(*inner, Type::Array(Box::new(base_typ), size)),
    }
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
        Token::LogicalAnd => BinaryOp::LogicalAnd,
        Token::LogicalOr => BinaryOp::LogicalOr,
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
        Token::Bang => UnaryOp::LogicalNot,
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
