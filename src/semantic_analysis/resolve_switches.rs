use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;

struct Context {
    cases: Vec<i32>,
    has_default: bool,
}

pub fn resolve_program(program: Program) -> ValidationResult<Program> {
    let program = match program {
        Program::FuncDef(func_def) => Program::FuncDef(resolve_func_def(func_def)?),
    };
    Ok(program)
}

fn resolve_func_def(func_def: FuncDef) -> ValidationResult<FuncDef> {
    let func_def = match func_def {
        FuncDef::Function(name, body) => {
            let body = body
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, &mut None))
                .collect::<ValidationResult<_>>()?;

            FuncDef::Function(name, body)
        }
    };
    Ok(func_def)
}

fn resolve_block_item(
    block_item: BlockItem,
    context: &mut Option<&mut Context>,
) -> ValidationResult<BlockItem> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, context)?),
        decl => decl,
    };
    Ok(block_item)
}

fn resolve_stmt(stmt: Stmt, context: &mut Option<&mut Context>) -> ValidationResult<Stmt> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(resolve_stmt(*then, context)?),
            Some(Box::new(resolve_stmt(*else_, context)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(cond, Box::new(resolve_stmt(*then, context)?), None),
        Stmt::Labelled(Label::Case(exp), stmt) => {
            if let Some(context) = context.as_mut() {
                let Exp::Const(case) = exp else {
                    return Err(ValidationError::NonConstCase(exp));
                };
                if context.cases.contains(&case) {
                    return Err(ValidationError::DuplicateCaseValue(case));
                }
                context.cases.push(case);
            } else {
                return Err(ValidationError::CaseOutsideLoop);
            }
            let stmt = Box::new(resolve_stmt(*stmt, context)?);
            Stmt::Labelled(Label::Default, stmt)
        }
        Stmt::Labelled(Label::Default, stmt) => {
            if let Some(context) = context.as_mut() {
                if !context.has_default {
                    context.has_default = true;
                } else {
                    return Err(ValidationError::DuplicateDefault);
                }
            } else {
                return Err(ValidationError::DefaultOutsideLoop);
            }
            let stmt = Box::new(resolve_stmt(*stmt, context)?);
            Stmt::Labelled(Label::Default, stmt)
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
        Stmt::Switch(cond, body, label) => {
            let mut new_context = Context {
                cases: Vec::new(),
                has_default: false,
            };
            let body = Box::new(resolve_stmt(*body, &mut Some(&mut new_context))?);
            Stmt::SwitchResolved(
                cond,
                body,
                label,
                new_context.cases,
                new_context.has_default,
            )
        }
        other => other,
    };
    Ok(stmt)
}
