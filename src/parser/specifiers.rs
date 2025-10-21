use crate::models::ast::StorageClass;
use crate::models::symbol::Type;
use crate::models::token::Token;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

pub enum Specifier {
    Int,
    Long,
    Static,
    Extern,
}

pub fn type_specifiers<'tokens, I>() -> impl Parser<'tokens, I, Type, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::KwInt => Type::Int,
        Token::KwLong => Type::Long,
    }
    .repeated()
    .at_least(1)
    .collect()
    .try_map(parse_type)
}

fn parse_type<'tokens>(specs: Vec<Type>, span: SimpleSpan) -> Result<Type, Rich<'tokens, Token>> {
    if specs == vec![Type::Int] {
        Ok(Type::Int)
    } else if specs == vec![Type::Int, Type::Long]
        || specs == vec![Type::Long, Type::Int]
        || specs == vec![Type::Long]
    {
        Ok(Type::Long)
    } else {
        Err(Rich::custom(span, "Invalid type specifier"))
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
            Specifier::Int => types.push(Type::Int),
            Specifier::Long => types.push(Type::Long),
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
