mod decl;
mod exp;
mod specifiers;
mod stmt;
mod utils;

use crate::models::ast::*;
use crate::models::token::Token;
use crate::utils::Ident;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use decl::decl;
use decl::var_decl;
use exp::exp;
use specifiers::specifiers;
use specifiers::type_specifiers;
use stmt::stmt;
use utils::SemicolonExt;

type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

fn array_suffix<'tokens, I>() -> impl Parser<'tokens, I, isize, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let const_int = select! {
        Token::ConstInt(i) => i as isize,
        Token::ConstLong(i) => i as isize,
        Token::ConstUInt(i) => i as isize,
        Token::ConstULong(i) => i as isize,
    };
    const_int.delimited_by(just(Token::LBracket), just(Token::RBracket))
}

fn ident<'tokens, I>() -> impl Parser<'tokens, I, Ident, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! { Token::Ident(ident) => ident }
}

pub fn parser<'tokens, I>() -> impl Parser<'tokens, I, Program, ParserError<'tokens>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let mut block = Recursive::declare();

    let decl = decl(block.clone());
    let stmt = stmt(block.clone());

    let block_item = choice((stmt.map(BlockItem::S), decl.clone().map(BlockItem::D)));

    block.define(
        block_item
            .repeated()
            .collect()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .labelled("block")
            .as_context(),
    );

    let program = decl.repeated().collect().map(Program::Program);

    program.then_ignore(end())
}
