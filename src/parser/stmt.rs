use super::ParserError;
use super::SemicolonExt;
use super::exp;
use super::var_decl;
use crate::models::ast::*;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

pub fn stmt<'tokens, I>(
    block: impl Parser<'tokens, I, Vec<BlockItem>, ParserError<'tokens>> + Clone + 'tokens,
) -> impl Parser<'tokens, I, Stmt, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };
    let exp_paren = exp().delimited_by(just(Token::LParen), just(Token::RParen));

    let mut stmt = Recursive::declare();

    let for_init = choice((
        var_decl().map(ForInit::InitDecl),
        exp().semicolon().map(ForInit::InitExp),
        just(Token::Semicolon).to(ForInit::Null),
    ));

    let label = choice((
        ident.map(Label::Ident),
        just(Token::KwCase).ignore_then(exp()).map(Label::Case),
        just(Token::KwDefault).to(Label::Default),
    ));

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
                .then(exp().or_not())
                .semicolon()
                .then(exp().or_not())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt.clone())
        .map(|(((init, cond), post), body)| Stmt::For(init, cond, post, Box::new(body), None));

    let switch_stmt = just(Token::KwSwitch)
        .ignore_then(exp().delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(stmt.clone())
        .map(|(cond, body)| Stmt::Switch(cond, Box::new(body), None, None));

    stmt.define(
        choice((
            exp_stmt,
            return_stmt,
            if_stmt,
            labelled_stmt,
            goto_stmt,
            block.map(Stmt::Compound),
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

    stmt
}
