use super::ParserError;
use super::specifiers::type_specifiers;
use crate::models::ast::*;
use crate::models::token::Token;
use crate::models::typ::Type;
use crate::utils::make_temporary;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

fn binary(op: BinaryOp, left: Exp, right: Exp) -> Exp {
    Exp::Binary(op, Box::new(left), Box::new(right), None)
}

fn update(op: BinaryOp, left: Exp, right: Exp, postfix: bool) -> Exp {
    let ptr_name = make_temporary();
    let ptr = Exp::Var(ptr_name, None);
    let deref = Exp::Dereference(Box::new(ptr), None);
    let mut bindings = vec![(ptr_name, Exp::AddressOf(Box::new(left), None))];
    let old = if postfix {
        let old_name = make_temporary();
        bindings.push((old_name, deref.clone()));
        Exp::Var(old_name, None)
    } else {
        deref.clone()
    };
    let assignment = Exp::Assignment(
        Box::new(deref),
        Box::new(binary(op, old.clone(), right)),
        None,
    );
    let (effects, result) = if postfix {
        (vec![assignment], old)
    } else {
        (vec![], assignment)
    };
    Exp::Sequence(bindings, effects, Box::new(result))
}

pub fn exp<'tokens, I>() -> impl Parser<'tokens, I, Exp, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|exp| {
        let argument_list = exp.clone().separated_by(just(Token::Comma)).collect();

        let func_call = select! { Token::Ident(ident) => ident }
            .then(argument_list.delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(name, args)| Exp::FuncCall(name, args, None));

        let atom = choice((
            func_call,
            select! {
                Token::Ident(ident) => Exp::Var(ident, None),
                Token::ConstInt(i) => Exp::Const(Const::Int(i)),
                Token::ConstLong(i) => Exp::Const(Const::Long(i)),
                Token::ConstUInt(i) => Exp::Const(Const::UInt(i)),
                Token::ConstULong(i) => Exp::Const(Const::ULong(i)),
                Token::ConstDouble(i) => Exp::Const(Const::Double(i)),
            },
            exp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        atom.pratt((
            postfix(14, just(Token::DoublePlus), |v: Exp, _, _| {
                update(BinaryOp::Add, v, Exp::Const(Const::Int(1)), true)
            }),
            postfix(14, just(Token::DoubleHyphen), |v: Exp, _, _| {
                update(BinaryOp::Subtract, v, Exp::Const(Const::Int(1)), true)
            }),
            prefix(13, just(Token::DoublePlus), |_, v: Exp, _| {
                update(BinaryOp::Add, v, Exp::Const(Const::Int(1)), false)
            }),
            prefix(13, just(Token::DoubleHyphen), |_, v: Exp, _| {
                update(BinaryOp::Subtract, v, Exp::Const(Const::Int(1)), false)
            }),
            prefix(
                13,
                one_of([Token::Hyphen, Token::Tilde, Token::Bang]),
                |op, v, _| Exp::Unary(convert_unop(op), Box::new(v), None),
            ),
            prefix(13, cast_op(), |typ, v, _| Exp::Cast(typ, Box::new(v))),
            prefix(13, just(Token::Ampersand), |_, v, _| {
                Exp::AddressOf(Box::new(v), None)
            }),
            prefix(13, just(Token::Star), |_, v, _| {
                Exp::Dereference(Box::new(v), None)
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
                    Exp::Conditional(Box::new(cond), Box::new(then), Box::new(else_), None)
                },
            ),
            infix(right(1), just(Token::Equal), |left, _, right, _| {
                Exp::Assignment(Box::new(left), Box::new(right), None)
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
}

fn abs_declarator<'tokens, I>()
-> impl Parser<'tokens, I, AbsDeclarator, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|abs_declarator| {
        choice((
            just(Token::Star)
                .ignore_then(abs_declarator.clone())
                .map(|abs_declarator| AbsDeclarator::Pointer(Box::new(abs_declarator))),
            abs_declarator.delimited_by(just(Token::LParen), just(Token::RParen)),
            empty().to(AbsDeclarator::Base),
        ))
    })
}

fn process_abs_declarator(abs_declarator: AbsDeclarator, base_typ: Type) -> Option<Type> {
    use AbsDeclarator::*;
    match abs_declarator {
        Base => Some(base_typ),
        Pointer(abs_declarator) => Some(Type::Pointer(Box::new(process_abs_declarator(
            *abs_declarator,
            base_typ,
        )?))),
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
