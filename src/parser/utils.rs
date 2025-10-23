use super::ParserError;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

pub trait SemicolonExt<'tokens, I, T>: Parser<'tokens, I, T, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    fn semicolon(self) -> impl Parser<'tokens, I, T, ParserError<'tokens>> + Clone {
        self.then_ignore(just(Token::Semicolon))
    }
}

impl<'tokens, I, T, P> SemicolonExt<'tokens, I, T> for P
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
    P: Parser<'tokens, I, T, ParserError<'tokens>> + Clone,
{
}
