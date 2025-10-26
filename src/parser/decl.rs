use super::Ident;
use super::ParserError;
use super::SemicolonExt;
use super::exp;
use super::specifiers;
use crate::models::ast::*;
use crate::models::token::Token;
use crate::models::typ::Type;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use specifiers::type_specifiers;

#[derive(Debug, Clone)]
enum Declarator {
    Ident(Ident),
    Pointer(Box<Declarator>),
    Func(Vec<(Type, Declarator)>, Box<Declarator>),
}

fn declarator<'tokens, I>() -> impl Parser<'tokens, I, Declarator, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };

    recursive(|declarator| {
        let simple_declarator = choice((
            ident.map(Declarator::Ident),
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

        let direct_declarator =
            simple_declarator
                .then(param_list.or_not())
                .map(|(simple, params)| match params {
                    Some(params) => Declarator::Func(params, Box::new(simple)),
                    None => simple,
                });

        choice((
            just(Token::Star)
                .ignore_then(declarator)
                .map(|declarator| Declarator::Pointer(Box::new(declarator))),
            direct_declarator,
        ))
    })
}

fn process_declarator(declarator: Declarator, base_typ: Type) -> Option<(Ident, Type, Vec<Ident>)> {
    match declarator {
        Declarator::Ident(name) => Some((name, base_typ, vec![])),
        Declarator::Pointer(d) => {
            let derived_type = Type::Pointer(Box::new(base_typ));
            Some(process_declarator(*d, derived_type)?)
        }
        Declarator::Func(params, d) => match *d {
            Declarator::Ident(name) => {
                let mut param_names = Vec::new();
                let mut param_types = Vec::new();
                for (p_base_type, p_declarator) in params {
                    let (param_name, param_type, _) =
                        process_declarator(p_declarator, p_base_type)?;
                    param_names.push(param_name);
                    param_types.push(param_type)
                }
                let derived_type = Type::Func(param_types, Box::new(base_typ));
                Some((name, derived_type, param_names))
            }
            _ => None,
        },
    }
}

pub fn var_decl<'tokens, I>() -> impl Parser<'tokens, I, VarDecl, ParserError<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    specifiers()
        .then(declarator())
        .then(just(Token::Equal).ignore_then(exp()).or_not())
        .semicolon()
        .try_map(|(((base_typ, storage_class), declarator), init), span| {
            let (name, typ, _params) =
                process_declarator(declarator, base_typ).ok_or(Rich::custom(span, ""))?;
            if matches!(typ, Type::Func(_, _)) {
                return Err(Rich::custom(span, ""));
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
        .try_map(|(((base_typ, storage_class), declarator), body), span| {
            let (name, typ, params) =
                process_declarator(declarator, base_typ).ok_or(Rich::custom(span, ""))?;
            if !matches!(typ, Type::Func(_, _)) {
                return Err(Rich::custom(span, ""));
            }
            Ok(FuncDecl::FuncDecl(name, params, body, typ, storage_class))
        })
        .labelled("function declaration")
        .as_context();

    choice((func_decl.map(Decl::FuncDecl), var_decl().map(Decl::VarDecl)))
}
