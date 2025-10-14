use crate::models::token::Token;
use chumsky::error::Rich;
use chumsky::prelude::*;
use chumsky::text;

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token>, extra::Err<Rich<'src, char>>> {
    let ident = text::ident().map(|s| match s {
        "int" => Token::KeywordInt,
        "return" => Token::KeywordReturn,
        "void" => Token::KeywordVoid,
        s => Token::Ident(s.to_string()),
    });

    let constant = text::int(10)
        .then_ignore(any().filter(|&c: &char| !c.is_alphanumeric()).rewind())
        .from_str()
        .unwrapped()
        .map(Token::Const);

    let three_chars_long = choice((
        just("<<=").to(Token::LShiftEqual),
        just(">>=").to(Token::RShiftEqual),
    ));

    let two_chars_long = choice((
        just("<<").to(Token::LShift),
        just(">>").to(Token::RShift),
        just("&&").to(Token::LogicalAnd),
        just("||").to(Token::LogicalOr),
        just("==").to(Token::DoubleEqual),
        just("!=").to(Token::BangEqual),
        just("<=").to(Token::LessOrEqual),
        just(">=").to(Token::GreaterOrEqual),
        just("+=").to(Token::PlusEqual),
        just("-=").to(Token::HyphenEqual),
        just("*=").to(Token::StarEqual),
        just("/=").to(Token::SlashEqual),
        just("%=").to(Token::PercentEqual),
        just("&=").to(Token::AndEqual),
        just("|=").to(Token::VBarEqual),
        just("^=").to(Token::CaretEqual),
        just("++").to(Token::DoublePlus),
        just("--").to(Token::DoubleHyphen),
    ));

    let one_char_long = choice((
        just("(").to(Token::LParen),
        just(")").to(Token::RParen),
        just("{").to(Token::LBrace),
        just("}").to(Token::RBrace),
        just(";").to(Token::Semicolon),
        just("+").to(Token::Plus),
        just("-").to(Token::Hyphen),
        just("*").to(Token::Star),
        just("/").to(Token::Slash),
        just("%").to(Token::Percent),
        just("~").to(Token::Tilde),
        just("!").to(Token::Bang),
        just("&").to(Token::Ampersand),
        just("|").to(Token::VerticalBar),
        just("^").to(Token::Caret),
        just("<").to(Token::LessThan),
        just(">").to(Token::GreaterThan),
        just("=").to(Token::Equal),
    ));

    choice((
        ident,
        constant,
        three_chars_long,
        two_chars_long,
        one_char_long,
    ))
    .padded()
    .repeated()
    .collect()
    .then_ignore(end())
}
