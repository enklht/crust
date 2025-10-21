use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

pub trait SemicolonExt<'token, I, T>:
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
