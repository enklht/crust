use super::Ident;
use super::ParserError;
use super::SemicolonExt;
use super::array_suffix;
use super::exp;
use super::ident;
use super::specifiers;
use crate::models::ast::*;
use crate::models::token::Token;
use crate::models::typ::{TypeId, TypeKind, Types};
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use specifiers::type_specifiers;

pub fn initializer<'tokens, I>()
-> impl Parser<'tokens, I, Initializer, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|initializer| {
        choice((
            exp().map(Initializer::Single),
            initializer
                .separated_by(just(Token::Comma))
                .at_least(1)
                .collect()
                .then_ignore(just(Token::Comma).or_not())
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .map(|inits| Initializer::Compound(None, inits)),
        ))
    })
}

pub fn var_decl<'tokens, I>() -> impl Parser<'tokens, I, VarDecl, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    specifiers()
        .then(declarator())
        .then(just(Token::Equal).ignore_then(initializer()).or_not())
        .semicolon()
        .try_map_with(|(((base_typ, storage_class), declarator), init), e| {
            let span = e.span();
            let types = &mut e.state().0;
            let (name, typ, _params) = process_declarator(declarator, base_typ, types)
                .ok_or(Rich::custom(span, "invalid declarator"))?;
            if matches!(types.kind(typ), TypeKind::Func(_, _)) {
                return Err(Rich::custom(
                    span,
                    "Expected a variable declaration, found a function declaration",
                ));
            }
            Ok(VarDecl::VarDecl(name, init, typ, storage_class))
        })
        .labelled("variable declaration")
        .as_context()
}

pub fn decl<'tokens, I>(
    block: impl Parser<'tokens, I, Vec<BlockItem>, ParserError<'tokens>> + Clone,
) -> impl Parser<'tokens, I, Decl, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let func_decl = specifiers()
        .then(declarator())
        .then(block.map(Some).or(just(Token::Semicolon).to(None)))
        .try_map_with(|(((base_typ, storage_class), declarator), body), e| {
            let span = e.span();
            let types = &mut e.state().0;
            let (name, typ, params) = process_declarator(declarator, base_typ, types)
                .ok_or(Rich::custom(span, "invalid declarator"))?;
            if !matches!(types.kind(typ), TypeKind::Func(_, _)) {
                return Err(Rich::custom(
                    span,
                    "Expected a function declaration before a function body",
                ));
            }
            Ok(FuncDecl::FuncDecl(name, params, body, typ, storage_class))
        })
        .labelled("function declaration")
        .as_context();

    choice((func_decl.map(Decl::FuncDecl), var_decl().map(Decl::VarDecl)))
}

#[derive(Clone)]
enum Declarator {
    Ident(Ident),
    Pointer(Box<Declarator>),
    Array(Box<Declarator>, usize),
    Func(Vec<(TypeId, Declarator)>, Box<Declarator>),
}

fn declarator<'tokens, I>() -> impl Parser<'tokens, I, Declarator, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|declarator| {
        let simple_declarator = choice((
            ident().map(Declarator::Ident),
            declarator
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        let param_list = choice((
            just(Token::KwVoid).to(Vec::new()),
            type_specifiers()
                .then(declarator.clone())
                .separated_by(just(Token::Comma))
                .collect(),
        ))
        .delimited_by(just(Token::LParen), just(Token::RParen));

        let direct_declarator = choice((
            simple_declarator
                .clone()
                .then(param_list)
                .map(|(simple, params)| Declarator::Func(params, Box::new(simple))),
            simple_declarator
                .clone()
                .foldl(array_suffix().repeated().at_least(1), |base_decl, dim| {
                    Declarator::Array(Box::new(base_decl), dim)
                }),
            simple_declarator,
        ));

        choice((
            just(Token::Star)
                .ignore_then(declarator)
                .map(|declarator| Declarator::Pointer(Box::new(declarator))),
            direct_declarator,
        ))
    })
}

fn process_declarator(
    declarator: Declarator,
    base_typ: TypeId,
    types: &mut Types,
) -> Option<(Ident, TypeId, Vec<Ident>)> {
    match declarator {
        Declarator::Ident(name) => Some((name, base_typ, vec![])),
        Declarator::Pointer(inner) => {
            let derived_type = types.intern(TypeKind::Pointer(base_typ));
            Some(process_declarator(*inner, derived_type, types)?)
        }
        Declarator::Array(inner, size) => {
            let derived_type = types.intern(TypeKind::Array(base_typ, size));
            process_declarator(*inner, derived_type, types)
        }
        Declarator::Func(params, inner) => match *inner {
            Declarator::Ident(name) => {
                let mut param_names = Vec::new();
                let mut param_types = Vec::new();
                for (p_base_type, p_declarator) in params {
                    let (param_name, param_type, _) =
                        process_declarator(p_declarator, p_base_type, types)?;
                    param_names.push(param_name);
                    param_types.push(param_type)
                }
                let derived_type = types.intern(TypeKind::Func(param_types, base_typ));
                Some((name, derived_type, param_names))
            }
            _ => None,
        },
    }
}
