use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::Type;
use crate::models::symbol::Types;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

#[derive(Debug)]
struct Context {
    cases: FxHashMap<Const, String>,
    default: Option<String>,
    typ: Type,
}

pub fn resolve_program(
    program: Program<Labelled>,
    types: &Types,
) -> ValidationResult<Program<SwitchResolved>> {
    let program = match program {
        Program::Program(func_decls) => Program::Program(
            func_decls
                .into_iter()
                .map(|decl| resolve_decl(decl, types))
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn resolve_decl(decl: Decl<Labelled>, types: &Types) -> ValidationResult<Decl<SwitchResolved>> {
    let decl = match decl {
        Decl::FuncDecl(func_decl) => Decl::FuncDecl(resolve_func_decl(func_decl, types)?),
        Decl::VarDecl(var_decl) => Decl::VarDecl(var_decl),
    };
    Ok(decl)
}

fn resolve_func_decl(
    func_def: FuncDecl<Labelled>,
    types: &Types,
) -> ValidationResult<FuncDecl<SwitchResolved>> {
    let func_decl = match func_def {
        FuncDecl::FuncDecl(name, params, Some(body), typ, storage_class) => {
            let body = body
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, &mut None, types))
                .collect::<ValidationResult<_>>()?;

            FuncDecl::FuncDecl(name, params, Some(body), typ, storage_class)
        }
        FuncDecl::FuncDecl(name, params, None, typ, storage_class) => {
            FuncDecl::FuncDecl(name, params, None, typ, storage_class)
        }
    };
    Ok(func_decl)
}

fn resolve_block_item(
    block_item: BlockItem<Labelled>,
    context: &mut Option<&mut Context>,
    types: &Types,
) -> ValidationResult<BlockItem<SwitchResolved>> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, context, types)?),
        BlockItem::D(decl) => BlockItem::D(resolve_decl(decl, types)?),
    };
    Ok(block_item)
}

fn resolve_stmt(
    stmt: Stmt<Labelled>,
    context: &mut Option<&mut Context>,
    types: &Types,
) -> ValidationResult<Stmt<SwitchResolved>> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(resolve_stmt(*then, context, types)?),
            Some(Box::new(resolve_stmt(*else_, context, types)?)),
        ),
        Stmt::If(cond, then, None) => {
            Stmt::If(cond, Box::new(resolve_stmt(*then, context, types)?), None)
        }
        Stmt::Labelled(Label::Case(exp), stmt) => {
            let label = make_label("case");
            if let Some(context) = context.as_mut() {
                let Exp::Const(c) = exp else {
                    return Err(ValidationError::NonConstCase(exp));
                };
                let c = match (&context.typ, c) {
                    (Type::Int, Const::Int(i)) => Const::Int(i),
                    (Type::Int, Const::Long(i)) => Const::Int(i as i32),
                    (Type::Long, Const::Int(i)) => Const::Long(i as i64),
                    (Type::Long, Const::Long(i)) => Const::Long(i),
                    _ => unreachable!(),
                };
                if context.cases.contains_key(&c) {
                    return Err(ValidationError::DuplicateCaseValue(c));
                }
                context.cases.insert(c, label.clone());
            } else {
                return Err(ValidationError::CaseOutsideSwitch);
            }

            let stmt = Box::new(resolve_stmt(*stmt, context, types)?);
            Stmt::Labelled(Label::Ident(label), stmt)
        }
        Stmt::Labelled(Label::Default, stmt) => {
            let label = make_label("default");
            if let Some(context) = context.as_mut() {
                match context.default {
                    Some(_) => return Err(ValidationError::DuplicateDefault),
                    None => context.default = Some(label.clone()),
                }
            } else {
                return Err(ValidationError::DefaultOutsideSwitch);
            }

            let stmt = Box::new(resolve_stmt(*stmt, context, types)?);
            Stmt::Labelled(Label::Ident(label), stmt)
        }
        Stmt::Labelled(label, body) => {
            let body = Box::new(resolve_stmt(*body, context, types)?);
            Stmt::Labelled(label, body)
        }
        Stmt::Compound(block) => {
            let block = block
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, context, types))
                .collect::<Result<_, _>>()?;
            Stmt::Compound(block)
        }
        Stmt::While(cond, body, label) => {
            let body = Box::new(resolve_stmt(*body, context, types)?);
            Stmt::While(cond, body, label)
        }
        Stmt::DoWhile(body, cond, label) => {
            let body = Box::new(resolve_stmt(*body, context, types)?);
            Stmt::DoWhile(body, cond, label)
        }
        Stmt::For(init, cond, post, body, label) => {
            let body = Box::new(resolve_stmt(*body, context, types)?);
            Stmt::For(init, cond, post, body, label)
        }
        Stmt::Switch(cond, body, label, ()) => {
            let mut new_context = Context {
                cases: FxHashMap::default(),
                default: None,
                typ: types.get(&cond).unwrap().clone(),
            };
            let body = Box::new(resolve_stmt(*body, &mut Some(&mut new_context), types)?);
            Stmt::Switch(cond, body, label, (new_context.cases, new_context.default))
        }
        Stmt::Return(exp) => Stmt::Return(exp),
        Stmt::Goto(label) => Stmt::Goto(label),
        Stmt::Expression(exp) => Stmt::Expression(exp),
        Stmt::Break(label) => Stmt::Break(label),
        Stmt::Continue(label) => Stmt::Continue(label),
        Stmt::Null => Stmt::Null,
    };
    Ok(stmt)
}
