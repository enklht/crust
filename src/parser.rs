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
use stmt::stmt;
use utils::SemicolonExt;

type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

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
