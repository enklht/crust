use crate::models::token::Token;
use chumsky::prelude::*;
use chumsky::text;

type Spanned<T> = (T, SimpleSpan);

pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token>>, extra::Err<Rich<'src, char>>> {
    let symbol = choice([
        just("<<=").to(Token::LShiftEqual),
        just(">>=").to(Token::RShiftEqual),
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
        just("?").to(Token::Question),
        just(":").to(Token::Colon),
        just(",").to(Token::Comma),
        just("[").to(Token::LBracket),
        just("]").to(Token::RBracket),
    ])
    .labelled("operator or punctuation");

    let ident = text::ident()
        .labelled("identifier or keyword")
        .map(|s| match s {
            "int" => Token::KwInt,
            "long" => Token::KwLong,
            "unsigned" => Token::KwUnsigned,
            "signed" => Token::KwSigned,
            "double" => Token::KwDouble,
            "return" => Token::KwReturn,
            "void" => Token::KwVoid,
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "goto" => Token::KwGoto,
            "do" => Token::KwDo,
            "while" => Token::KwWhile,
            "for" => Token::KwFor,
            "break" => Token::KwBreak,
            "continue" => Token::KwContinue,
            "switch" => Token::KwSwitch,
            "case" => Token::KwCase,
            "default" => Token::KwDefault,
            "static" => Token::KwStatic,
            "extern" => Token::KwExtern,
            s => Token::Ident(Box::new(s.to_string()).leak()),
        });

    let const_int = text::int(10)
        .then_ignore(regex(r"[^\w\.]").rewind())
        .labelled("integer constant")
        .validate(|s: &str, e, emitter| {
            if let Ok(i) = s.parse::<i32>() {
                Token::ConstInt(i)
            } else if let Ok(i) = s.parse::<i64>() {
                Token::ConstLong(i)
            } else {
                emitter.emit(Rich::custom(
                    e.span(),
                    "Integer constant is too large for 'int' or 'long'",
                ));
                Token::ConstLong(0)
            }
        });

    let const_long = text::int(10)
        .then_ignore(one_of("lL"))
        .then_ignore(regex(r"[^\w\.]").rewind())
        .labelled("integer constant")
        .validate(|s: &str, e, emitter| {
            if let Ok(i) = s.parse::<i64>() {
                Token::ConstLong(i)
            } else {
                emitter.emit(Rich::custom(
                    e.span(),
                    "Integer constant is too large for 'long'",
                ));
                Token::ConstLong(0)
            }
        });

    let const_uint = text::int(10)
        .then_ignore(one_of("uU"))
        .then_ignore(regex(r"[^\w\.]").rewind())
        .labelled("integer constant")
        .validate(|s: &str, e, emitter| {
            if let Ok(i) = s.parse::<u32>() {
                Token::ConstUInt(i)
            } else if let Ok(i) = s.parse::<u64>() {
                Token::ConstULong(i)
            } else {
                emitter.emit(Rich::custom(
                    e.span(),
                    "Integer constant is too large for 'unsigned int' or 'unsigned long'",
                ));
                Token::ConstULong(0)
            }
        });

    let const_ulong = text::int(10)
        .then_ignore(regex("[lL][uU]|[uU][lL]"))
        .then_ignore(regex(r"[^\w\.]").rewind())
        .labelled("integer constant")
        .validate(|s: &str, e, emitter| {
            if let Ok(i) = s.parse::<u64>() {
                Token::ConstULong(i)
            } else {
                emitter.emit(Rich::custom(
                    e.span(),
                    "Integer constant is too large for 'unsigned long'",
                ));
                Token::ConstULong(0)
            }
        });

    let const_double = regex(r"((\d*\.\d+|\d+\.?)[Ee][+-]?\d+|\d*\.\d+|\d+\.)")
        .then_ignore(regex(r"[^\w\.]").rewind())
        .labelled("floating-point constant")
        .validate(|s: &str, e, emitter| {
            if let Ok(i) = s.parse::<f64>() {
                Token::ConstDouble(i)
            } else {
                emitter.emit(Rich::custom(e.span(), "Invalid floating-point constant"));
                Token::ConstDouble(0.0)
            }
        });

    choice((
        symbol,
        ident,
        const_int,
        const_long,
        const_uint,
        const_ulong,
        const_double,
    ))
    .map_with(|t, e| (t, e.span()))
    .padded()
    .repeated()
    .collect()
    .then_ignore(end())
}
