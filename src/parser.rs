use crate::models::ast::*;
use crate::models::symbol::Type;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

macro_rules! ParserOutput {
    ($output:ty) => {
        impl Parser<'tokens, I, $output, ParserError<'tokens>> + Clone
    };
}

trait SemicolonExt<'token, I, T>:
    Parser<'token, I, T, extra::Err<Rich<'token, Token>>> + Clone
where
    I: ValueInput<'token, Token = Token, Span = SimpleSpan>,
{
    fn semicolon(self) -> impl Parser<'token, I, T, extra::Err<Rich<'token, Token>>> + Clone {
        self.then_ignore(just(Token::Semicolon))
    }
}

impl<'token, I, T, P> SemicolonExt<'token, I, T> for P
where
    I: ValueInput<'token, Token = Token, Span = SimpleSpan>,
    P: Parser<'token, I, T, extra::Err<Rich<'token, Token>>> + Clone,
{
}

enum Specifier {
    Int,
    Long,
    Static,
    Extern,
}

#[inline]
fn parse_type<'tokens>(specs: Vec<Type>, span: SimpleSpan) -> Result<Type, Rich<'tokens, Token>> {
    if specs == vec![Type::Int] {
        Ok(Type::Int)
    } else if specs == vec![Type::Int, Type::Long]
        || specs == vec![Type::Long, Type::Int]
        || specs == vec![Type::Long]
    {
        Ok(Type::Long)
    } else {
        Err(Rich::custom(span, "Invalid type specifier"))
    }
}

fn parse_specs<'tokens>(
    specs: Vec<Specifier>,
    span: SimpleSpan,
) -> Result<(Type, StorageClass), Rich<'tokens, Token>> {
    let mut types = Vec::new();
    let mut storage_classes = Vec::new();

    for spec in specs {
        match spec {
            Specifier::Int => types.push(Type::Int),
            Specifier::Long => types.push(Type::Long),
            Specifier::Static => storage_classes.push(StorageClass::Static),
            Specifier::Extern => storage_classes.push(StorageClass::Extern),
        }
    }

    let typ = parse_type(types, span)?;

    let storage_class = if storage_classes.len() > 1 {
        return Err(Rich::custom(span, "Invalid storage class"));
    } else {
        storage_classes.pop().unwrap_or(StorageClass::None)
    };

    Ok((typ, storage_class))
}

fn type_specifiers<'tokens, I>() -> impl Parser<'tokens, I, Type, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::KwInt => Type::Int,
        Token::KwLong => Type::Long,
    }
    .repeated()
    .at_least(1)
    .collect()
    .try_map(parse_type)
}

pub fn parser<'tokens, I>() -> impl Parser<'tokens, I, Program<Plain>, ParserError<'tokens>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };

    let exp = exp();
    let exp_paren = exp
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let type_specifiers = type_specifiers();

    let specifiers = select! {
        Token::KwInt => Specifier::Int,
        Token::KwLong => Specifier::Long,
        Token::KwStatic => Specifier::Static,
        Token::KwExtern => Specifier::Extern,
    }
    .repeated()
    .at_least(1)
    .collect()
    .try_map(parse_specs);

    let var_decl = specifiers
        .then(ident)
        .then(just(Token::Equal).ignore_then(exp.clone()).or_not())
        .semicolon()
        .map(|(((typ, storage_class), name), init)| {
            VarDecl::VarDecl(name, init, typ, storage_class)
        })
        .labelled("declaration")
        .as_context();

    let for_init = choice((
        var_decl.clone().map(ForInit::InitDecl),
        exp.clone()
            .then_ignore(just(Token::Semicolon))
            .map(ForInit::InitExp),
        just(Token::Semicolon).to(ForInit::Null),
    ));

    let label = choice((
        ident.map(Label::Ident),
        just(Token::KwCase)
            .ignore_then(exp.clone())
            .map(Label::Case),
        just(Token::KwDefault).to(Label::Default),
    ));

    let mut stmt = Recursive::declare();
    let mut block = Recursive::declare();
    let mut decl = Recursive::declare();

    let exp_stmt = exp.clone().semicolon().map(Stmt::Expression);

    let return_stmt = just(Token::KwReturn)
        .ignore_then(exp.clone())
        .semicolon()
        .map(Stmt::Return);

    let if_stmt = just(Token::KwIf)
        .ignore_then(exp_paren.clone())
        .then(stmt.clone())
        .then(
            just(Token::KwElse)
                .ignore_then(stmt.clone())
                .map(Box::new)
                .or_not(),
        )
        .map(|((cond, then), else_)| Stmt::If(cond, Box::new(then), else_));

    let labelled_stmt = label
        .then_ignore(just(Token::Colon))
        .then(stmt.clone())
        .map(|(label, stmt)| Stmt::Labelled(label, Box::new(stmt)));

    let goto_stmt = just(Token::KwGoto)
        .ignore_then(ident)
        .semicolon()
        .map(Stmt::Goto);

    let while_stmt = just(Token::KwWhile)
        .ignore_then(exp_paren.clone())
        .then(stmt.clone())
        .map(|(cond, body)| Stmt::While(cond, Box::new(body), ()));

    let do_while_stmt = just(Token::KwDo)
        .ignore_then(stmt.clone())
        .then_ignore(just(Token::KwWhile))
        .then(exp_paren.clone())
        .semicolon()
        .map(|(body, cond)| Stmt::DoWhile(Box::new(body), cond, ()));

    let for_stmt = just(Token::KwFor)
        .ignore_then(
            for_init
                .then(exp.clone().or_not())
                .semicolon()
                .then(exp.clone().or_not())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(((init, cond), post), body)| Stmt::For(init, cond, post, Box::new(body), ()));

    let switch_stmt = just(Token::KwSwitch)
        .ignore_then(
            exp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(cond, body)| Stmt::Switch(cond, Box::new(body), (), ()));

    stmt.define(
        choice((
            exp_stmt,
            return_stmt,
            if_stmt,
            labelled_stmt,
            goto_stmt,
            block.clone().map(Stmt::Compound),
            just(Token::KwBreak).semicolon().to(Stmt::Break(())),
            just(Token::KwContinue).semicolon().to(Stmt::Continue(())),
            while_stmt,
            do_while_stmt,
            for_stmt,
            switch_stmt,
            just(Token::Semicolon).to(Stmt::Null),
        ))
        .labelled("statement")
        .as_context(),
    );

    let block_item = choice((stmt.map(BlockItem::S), decl.clone().map(BlockItem::D)));

    block.define(
        block_item
            .repeated()
            .collect()
            .delimited_by(just(Token::LBrace), just(Token::RBrace)),
    );

    let param_list = choice((
        just(Token::KwVoid).to(Vec::new()),
        type_specifiers
            .then(ident)
            .separated_by(just(Token::Comma))
            .collect(),
    ));

    let func_decl = specifiers
        .then(ident)
        .then(param_list.delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block.map(Some).or(just(Token::Semicolon).to(None)))
        .map(|((((typ, storage_class), name), params), body)| {
            let (param_types, params): (Vec<_>, Vec<_>) = params.into_iter().unzip();
            let typ = Type::Func(param_types, Box::new(typ));
            FuncDecl::FuncDecl(name, params, body, typ, storage_class)
        });

    decl.define(choice((
        func_decl.clone().map(Decl::FuncDecl),
        var_decl.map(Decl::VarDecl),
    )));

    let program = decl.repeated().collect().map(Program::Program);

    program.then_ignore(end())
}

fn exp<'tokens, I>() -> impl Parser<'tokens, I, Exp, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|exp| {
        let argument_list = exp.clone().separated_by(just(Token::Comma)).collect();

        let func_call = select! { Token::Ident(ident) => ident }
            .then(argument_list.delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(name, args)| Exp::FuncCall(name, args));

        let atom = choice((
            func_call,
            select! {
                Token::Ident(ident) => Exp::Var(ident.clone()),
                Token::ConstInt(i) => Exp::Const(Const::Int(i)),
                Token::ConstLong(i) => Exp::Const(Const::Long(i)),
            },
            exp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        atom.pratt((
            postfix(
                14,
                one_of([Token::DoublePlus, Token::DoubleHyphen]),
                |v, op, _| Exp::Unary(convert_unop(op), Box::new(v)),
            ),
            prefix(13, just(Token::DoublePlus), |_, v: Exp, _| {
                Exp::Assignment(
                    Box::new(v.clone()),
                    Box::new(Exp::Binary(
                        BinaryOp::Add,
                        Box::new(v),
                        Box::new(Exp::Const(Const::Int(1))),
                    )),
                )
            }),
            prefix(13, just(Token::DoubleHyphen), |_, v: Exp, _| {
                Exp::Assignment(
                    Box::new(v.clone()),
                    Box::new(Exp::Binary(
                        BinaryOp::Subtract,
                        Box::new(v),
                        Box::new(Exp::Const(Const::Int(1))),
                    )),
                )
            }),
            prefix(
                13,
                one_of([Token::Hyphen, Token::Tilde, Token::Bang]),
                |op, v, _| Exp::Unary(convert_unop(op), Box::new(v)),
            ),
            prefix(
                13,
                type_specifiers().delimited_by(just(Token::LParen), just(Token::RParen)),
                |typ, v, _| Exp::Cast(typ, Box::new(v)),
            ),
            infix(
                left(12),
                one_of([Token::Star, Token::Slash, Token::Percent]),
                |left, op, right, _| {
                    Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(11),
                one_of([Token::Plus, Token::Hyphen]),
                |left, op, right, _| {
                    Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(10),
                one_of([Token::LShift, Token::RShift]),
                |left, op, right, _| {
                    Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
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
                    Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(
                left(8),
                one_of([Token::DoubleEqual, Token::BangEqual]),
                |left, op, right, _| {
                    Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
                },
            ),
            infix(left(7), just(Token::Ampersand), |left, op, right, _| {
                Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(6), just(Token::Caret), |left, op, right, _| {
                Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(5), just(Token::VerticalBar), |left, op, right, _| {
                Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(4), just(Token::LogicalAnd), |left, op, right, _| {
                Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(left(3), just(Token::LogicalOr), |left, op, right, _| {
                Exp::Binary(convert_binop(op), Box::new(left), Box::new(right))
            }),
            infix(
                right(2),
                exp.delimited_by(just(Token::Question), just(Token::Colon)),
                |cond, then, else_, _| {
                    Exp::Conditional(Box::new(cond), Box::new(then), Box::new(else_))
                },
            ),
            infix(right(1), just(Token::Equal), |left, _, right, _| {
                Exp::Assignment(Box::new(left), Box::new(right))
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
                |left: Exp, op, right, _| {
                    let op = convert_compound(op);
                    Exp::Assignment(
                        Box::new(left.clone()),
                        Box::new(Exp::Binary(op, Box::new(left), Box::new(right))),
                    )
                },
            ),
        ))
    })
    .labelled("expession")
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
