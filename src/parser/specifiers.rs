use super::ParserError;
use crate::models::ast::StorageClass;
use crate::models::token::Token;
use crate::models::typ::Type;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

#[derive(Debug, PartialEq)]
enum Specifier {
    Int,
    Long,
    Unsigned,
    Signed,
    Double,
    Static,
    Extern,
}

pub fn type_specifiers<'tokens, I>() -> impl Parser<'tokens, I, Type, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::KwInt => Specifier::Int,
        Token::KwLong => Specifier::Long,
        Token::KwUnsigned => Specifier::Unsigned,
        Token::KwSigned => Specifier::Signed,
        Token::KwDouble => Specifier::Double,
    }
    .repeated()
    .collect()
    .try_map(parse_type)
}

fn parse_type<'tokens>(
    specs: Vec<Specifier>,
    span: SimpleSpan,
) -> Result<Type, Rich<'tokens, Token>> {
    if specs == vec![Specifier::Double] {
        return Ok(Type::Double);
    } else if specs.contains(&Specifier::Double) {
        return Err(Rich::custom(span, "Cannot combine 'double' with others"));
    }

    const INT: u32 = 1 << 0;
    const LONG: u32 = 1 << 1;
    const UNSIGNED: u32 = 1 << 2;
    const SIGNED: u32 = 1 << 3;

    let mut flags = 0u32;

    for spec in specs {
        use Specifier::*;
        let mask = match spec {
            Int => INT,
            Long => LONG,
            Unsigned => UNSIGNED,
            Signed => SIGNED,
            _ => unreachable!(),
        };

        if flags & mask != 0 {
            return Err(Rich::custom(span, "duplicate type specifier"));
        }
        flags |= mask;

        if (flags & UNSIGNED != 0) && (flags & SIGNED != 0) {
            return Err(Rich::custom(span, "cannot combine unsigned with signed"));
        }
    }

    if (flags & UNSIGNED != 0) && (flags & LONG != 0) {
        Ok(Type::ULong)
    } else if flags & UNSIGNED != 0 {
        Ok(Type::UInt)
    } else if flags & LONG != 0 {
        Ok(Type::Long)
    } else if flags != 0 {
        Ok(Type::Int)
    } else {
        Err(Rich::custom(span, "invalid type specifier"))
    }
}

pub fn specifiers<'tokens, I>()
-> impl Parser<'tokens, I, (Type, StorageClass), ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::KwInt => Specifier::Int,
        Token::KwLong => Specifier::Long,
        Token::KwUnsigned => Specifier::Unsigned,
        Token::KwSigned => Specifier::Signed,
        Token::KwDouble => Specifier::Double,
        Token::KwStatic => Specifier::Static,
        Token::KwExtern => Specifier::Extern,
    }
    .repeated()
    .at_least(1)
    .collect()
    .try_map(parse_specs)
}

fn parse_specs<'tokens>(
    specs: Vec<Specifier>,
    span: SimpleSpan,
) -> Result<(Type, StorageClass), Rich<'tokens, Token>> {
    let mut types = Vec::new();
    let mut storage_classes = Vec::new();

    for spec in specs {
        match spec {
            Specifier::Int => types.push(Specifier::Int),
            Specifier::Long => types.push(Specifier::Long),
            Specifier::Unsigned => types.push(Specifier::Unsigned),
            Specifier::Signed => types.push(Specifier::Signed),
            Specifier::Double => types.push(Specifier::Double),
            Specifier::Static => storage_classes.push(StorageClass::Static),
            Specifier::Extern => storage_classes.push(StorageClass::Extern),
        }
    }

    let typ = parse_type(types, span)?;

    let storage_class = if storage_classes.len() > 1 {
        return Err(Rich::custom(span, "Invalid storage class"));
    } else {
        storage_classes.pop().unwrap_or(StorageClass::None)
    };

    Ok((typ, storage_class))
}
