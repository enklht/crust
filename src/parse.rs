use crate::models::ast::*;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

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

pub fn parser<'tokens, I>()
-> impl Parser<'tokens, I, Program<Plain>, extra::Err<Rich<'tokens, Token>>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };

    let exp = exp();
    let exp_paren = exp
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let var_decl = just(Token::KwInt)
        .ignore_then(ident)
        .then(just(Token::Equal).ignore_then(exp.clone()).or_not())
        .semicolon()
        .map(|(name, init)| VarDecl::VarDecl(name, init))
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
        .map(|(cond, body)| Stmt::While(cond, Box::new(body), None));

    let do_while_stmt = just(Token::KwDo)
        .ignore_then(stmt.clone())
        .then_ignore(just(Token::KwWhile))
        .then(exp_paren.clone())
        .semicolon()
        .map(|(body, cond)| Stmt::DoWhile(Box::new(body), cond, None));

    let for_stmt = just(Token::KwFor)
        .ignore_then(
            for_init
                .then(exp.clone().or_not())
                .semicolon()
                .then(exp.clone().or_not())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(((init, cond), post), body)| Stmt::For(init, cond, post, Box::new(body), None));

    let switch_stmt = just(Token::KwSwitch)
        .ignore_then(
            exp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(cond, body)| Stmt::Switch(cond, Box::new(body), None, ()));

    stmt.define(
        choice((
            exp_stmt,
            return_stmt,
            if_stmt,
            labelled_stmt,
            goto_stmt,
            block.clone().map(Stmt::Compound),
            just(Token::KwBreak).semicolon().to(Stmt::Break(None)),
            just(Token::KwContinue).semicolon().to(Stmt::Continue(None)),
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
        just(Token::KwInt)
            .ignore_then(ident)
            .separated_by(just(Token::Comma))
            .collect(),
    ));

    let func_decl = just(Token::KwInt)
        .ignore_then(ident)
        .then(param_list.delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block.map(Some).or(just(Token::Semicolon).to(None)))
        .map(|((name, params), body)| FuncDecl::FuncDecl(name, params, body));

    decl.define(choice((
        func_decl.clone().map(Decl::FuncDecl),
        var_decl.map(Decl::VarDecl),
    )));

    let program = func_decl.repeated().collect().map(Program::Program);

    program.then_ignore(end())
}

fn exp<'tokens, I>() -> impl Parser<'tokens, I, Exp, extra::Err<Rich<'tokens, Token>>> + Clone
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
                Token::Const(i) => Exp::Const(i)
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
                        Box::new(Exp::Const(1)),
                    )),
                )
            }),
            prefix(13, just(Token::DoubleHyphen), |_, v: Exp, _| {
                Exp::Assignment(
                    Box::new(v.clone()),
                    Box::new(Exp::Binary(
                        BinaryOp::Subtract,
                        Box::new(v),
                        Box::new(Exp::Const(1)),
                    )),
                )
            }),
            prefix(
                13,
                one_of([Token::Hyphen, Token::Tilde, Token::Bang]),
                |op, v, _| Exp::Unary(convert_unop(op), Box::new(v)),
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
