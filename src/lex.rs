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

    let punctuation = choice((
        just("(").to(Token::LParen),
        just(")").to(Token::RParen),
        just("{").to(Token::LBrace),
        just("}").to(Token::RBrace),
        just(";").to(Token::Semicolon),
    ));

    let operator = choice((
        just("+").to(Token::Plus),
        just("-").to(Token::Hyphen),
        just("*").to(Token::Star),
        just("/").to(Token::Slash),
        just("%").to(Token::Percent),
        just("~").to(Token::Tilde),
        just("&").to(Token::Ampersand),
        just("|").to(Token::VerticalBar),
        just("^").to(Token::Caret),
        just("<<").to(Token::LShift),
        just(">>").to(Token::RShift),
        just("!").to(Token::Bang),
        just("&&").to(Token::LogicalAnd),
        just("||").to(Token::LogicalOr),
    ));

    let relational = choice((
        just("==").to(Token::DoubleEqual),
        just("!=").to(Token::BangEqual),
        just("<").to(Token::LessThan),
        just("<=").to(Token::LessOrEqual),
        just(">").to(Token::GreaterThan),
        just(">=").to(Token::GreaterOrEqual),
    ));

    let assignment = choice((
        just("=").to(Token::Equal),
        just("+=").to(Token::PlusEqual),
        just("-=").to(Token::HyphenEqual),
        just("*=").to(Token::StarEqual),
        just("/=").to(Token::SlashEqual),
        just("%=").to(Token::PercentEqual),
        just("&=").to(Token::AndEqual),
        just("|=").to(Token::VBarEqual),
        just("^=").to(Token::CaretEqual),
        just("<<=").to(Token::LShiftEqual),
        just(">>=").to(Token::RShiftEqual),
        just("++").to(Token::DoublePlus),
        just("--").to(Token::DoubleHyphen),
    ));

    choice((
        ident,
        constant,
        punctuation,
        operator,
        relational,
        assignment,
    ))
    .padded()
    .repeated()
    .collect()
    .then_ignore(end())
}
