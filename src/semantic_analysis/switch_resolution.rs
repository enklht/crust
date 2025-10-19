use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

struct Context {
    cases: FxHashMap<i32, String>,
    default: Option<String>,
}

pub fn resolve_program(program: Program<Labelled>) -> ValidationResult<Program<SwitchResolved>> {
    let program = match program {
        Program::Program(func_decls) => Program::Program(
            func_decls
                .into_iter()
                .map(resolve_decl)
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn resolve_decl(decl: Decl<Labelled>) -> ValidationResult<Decl<SwitchResolved>> {
    let decl = match decl {
        Decl::FuncDecl(func_decl) => Decl::FuncDecl(resolve_func_decl(func_decl)?),
        Decl::VarDecl(var_decl) => Decl::VarDecl(var_decl),
    };
    Ok(decl)
}

fn resolve_func_decl(func_def: FuncDecl<Labelled>) -> ValidationResult<FuncDecl<SwitchResolved>> {
    let func_decl = match func_def {
        FuncDecl::FuncDecl(name, params, Some(body), storage_class) => {
            let body = body
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, &mut None))
                .collect::<ValidationResult<_>>()?;

            FuncDecl::FuncDecl(name, params, Some(body), storage_class)
        }
        FuncDecl::FuncDecl(name, params, None, storage_class) => {
            FuncDecl::FuncDecl(name, params, None, storage_class)
        }
    };
    Ok(func_decl)
}

fn resolve_block_item(
    block_item: BlockItem<Labelled>,
    context: &mut Option<&mut Context>,
) -> ValidationResult<BlockItem<SwitchResolved>> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, context)?),
        BlockItem::D(decl) => BlockItem::D(resolve_decl(decl)?),
    };
    Ok(block_item)
}

fn resolve_stmt(
    stmt: Stmt<Labelled>,
    context: &mut Option<&mut Context>,
) -> ValidationResult<Stmt<SwitchResolved>> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(resolve_stmt(*then, context)?),
            Some(Box::new(resolve_stmt(*else_, context)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(cond, Box::new(resolve_stmt(*then, context)?), None),
        Stmt::Labelled(Label::Case(exp), stmt) => {
            let label = make_label("case");
            if let Some(context) = context.as_mut() {
                let Exp::Const(case) = exp else {
                    return Err(ValidationError::NonConstCase(exp));
                };
                if context.cases.contains_key(&case) {
                    return Err(ValidationError::DuplicateCaseValue(case));
                }
                context.cases.insert(case, label.clone());
            } else {
                return Err(ValidationError::CaseOutsideSwitch);
            }

            let stmt = Box::new(resolve_stmt(*stmt, context)?);
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

            let stmt = Box::new(resolve_stmt(*stmt, context)?);
            Stmt::Labelled(Label::Ident(label), stmt)
        }
        Stmt::Labelled(label, body) => {
            let body = Box::new(resolve_stmt(*body, context)?);
            Stmt::Labelled(label, body)
        }
        Stmt::Compound(block) => {
            let block = block
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, context))
                .collect::<Result<_, _>>()?;
            Stmt::Compound(block)
        }
        Stmt::While(cond, body, label) => {
            let body = Box::new(resolve_stmt(*body, context)?);
            Stmt::While(cond, body, label)
        }
        Stmt::DoWhile(body, cond, label) => {
            let body = Box::new(resolve_stmt(*body, context)?);
            Stmt::DoWhile(body, cond, label)
        }
        Stmt::For(init, cond, post, body, label) => {
            let body = Box::new(resolve_stmt(*body, context)?);
            Stmt::For(init, cond, post, body, label)
        }
        Stmt::Switch(cond, body, label, ()) => {
            let mut new_context = Context {
                cases: FxHashMap::default(),
                default: None,
            };
            let body = Box::new(resolve_stmt(*body, &mut Some(&mut new_context))?);
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
