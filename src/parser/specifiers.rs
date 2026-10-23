use super::ParserError;
use crate::models::ast::StorageClass;
use crate::models::token::Token;
use crate::models::typ::{TypeId, Types};
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

pub fn type_specifiers<'tokens, I>() -> impl Parser<'tokens, I, TypeId, ParserError<'tokens>> + Clone
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
    .labelled("type specifier")
    .repeated()
    .at_least(1)
    .collect()
    .validate(
        |specs, e: &mut chumsky::input::MapExtra<'tokens, '_, I, ParserError<'tokens>>, emitter| {
            let span = e.span();
            let types = &e.state().0;
            parse_type(specs, span, types).unwrap_or_else(|err| {
                emitter.emit(err);
                types.int
            })
        },
    )
}

fn parse_type<'tokens>(
    specs: Vec<Specifier>,
    span: SimpleSpan,
    types: &Types,
) -> Result<TypeId, Rich<'tokens, Token>> {
    if specs == vec![Specifier::Double] {
        return Ok(types.double);
    } else if specs.contains(&Specifier::Double) {
        return Err(Rich::custom(
            span,
            "Cannot combine 'double' with other type specifiers",
        ));
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
            return Err(Rich::custom(span, "Duplicate type specifier"));
        }
        flags |= mask;

        if (flags & UNSIGNED != 0) && (flags & SIGNED != 0) {
            return Err(Rich::custom(
                span,
                "Cannot combine 'unsigned' with 'signed'",
            ));
        }
    }

    if (flags & UNSIGNED != 0) && (flags & LONG != 0) {
        Ok(types.ulong)
    } else if flags & UNSIGNED != 0 {
        Ok(types.uint)
    } else if flags & LONG != 0 {
        Ok(types.long)
    } else if flags != 0 {
        Ok(types.int)
    } else {
        Err(Rich::custom(span, "Expected a type specifier"))
    }
}

pub fn specifiers<'tokens, I>()
-> impl Parser<'tokens, I, (TypeId, StorageClass), ParserError<'tokens>> + Clone
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
    .labelled("declaration specifier")
    .repeated()
    .at_least(1)
    .collect()
    .validate(
        |specs, e: &mut chumsky::input::MapExtra<'tokens, '_, I, ParserError<'tokens>>, emitter| {
            let span = e.span();
            let types = &e.state().0;
            parse_specs(specs, span, types).unwrap_or_else(|err| {
                emitter.emit(err);
                (types.int, StorageClass::None)
            })
        },
    )
}

fn parse_specs<'tokens>(
    specs: Vec<Specifier>,
    span: SimpleSpan,
    types: &Types,
) -> Result<(TypeId, StorageClass), Rich<'tokens, Token>> {
    let mut type_specs = Vec::new();
    let mut storage_classes = Vec::new();

    for spec in specs {
        match spec {
            Specifier::Int => type_specs.push(Specifier::Int),
            Specifier::Long => type_specs.push(Specifier::Long),
            Specifier::Unsigned => type_specs.push(Specifier::Unsigned),
            Specifier::Signed => type_specs.push(Specifier::Signed),
            Specifier::Double => type_specs.push(Specifier::Double),
            Specifier::Static => storage_classes.push(StorageClass::Static),
            Specifier::Extern => storage_classes.push(StorageClass::Extern),
        }
    }

    let typ = parse_type(type_specs, span, types)?;

    let storage_class = if storage_classes.len() > 1 {
        return Err(Rich::custom(
            span,
            "A declaration can contain only one storage class: 'static' or 'extern'",
        ));
    } else {
        storage_classes.pop().unwrap_or(StorageClass::None)
    };

    Ok((typ, storage_class))
}
