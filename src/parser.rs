mod exp;
mod specifiers;
mod utils;

use crate::models::ast::*;
use crate::models::symbol::Type;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use exp::exp;
use specifiers::specifiers;
use specifiers::type_specifiers;
use utils::SemicolonExt;

type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

pub fn parser<'tokens, I>() -> impl Parser<'tokens, I, Program<Plain>, ParserError<'tokens>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };

    let exp_paren = exp().delimited_by(just(Token::LParen), just(Token::RParen));

    let var_decl = specifiers()
        .then(ident)
        .then(just(Token::Equal).ignore_then(exp()).or_not())
        .semicolon()
        .map(|(((typ, storage_class), name), init)| {
            VarDecl::VarDecl(name, init, typ, storage_class)
        })
        .labelled("variable declaration")
        .as_context();

    let for_init = choice((
        var_decl.clone().map(ForInit::InitDecl),
        exp()
            .then_ignore(just(Token::Semicolon))
            .map(ForInit::InitExp),
        just(Token::Semicolon).to(ForInit::Null),
    ));

    let label = choice((
        ident.map(Label::Ident),
        just(Token::KwCase).ignore_then(exp()).map(Label::Case),
        just(Token::KwDefault).to(Label::Default),
    ));

    let mut stmt = Recursive::declare();
    let mut block = Recursive::declare();
    let mut decl = Recursive::declare();

    let exp_stmt = exp().semicolon().map(Stmt::Expression);

    let return_stmt = just(Token::KwReturn)
        .ignore_then(exp())
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
                .then(exp().or_not())
                .semicolon()
                .then(exp().or_not())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(((init, cond), post), body)| Stmt::For(init, cond, post, Box::new(body), ()));

    let switch_stmt = just(Token::KwSwitch)
        .ignore_then(exp().delimited_by(just(Token::LParen), just(Token::RParen)))
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
        type_specifiers()
            .then(ident)
            .separated_by(just(Token::Comma))
            .collect(),
    ));

    let func_decl = specifiers()
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
