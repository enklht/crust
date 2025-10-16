use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use std::collections::HashMap;

pub fn resolve_program(program: Program) -> ValidationResult<Program> {
    Ok(match program {
        Program::FuncDef(func_def) => Program::FuncDef(resolve_func_def(func_def)?),
    })
}

fn resolve_func_def(func_def: FuncDef) -> ValidationResult<FuncDef> {
    match func_def {
        FuncDef::Function(name, body) => {
            let mut labels = HashMap::new();
            for block_item in &body {
                if let BlockItem::S(stmt) = block_item {
                    collect_labels(stmt, &mut labels)?;
                }
            }

            let body = body
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, &labels))
                .collect::<ValidationResult<_>>()?;

            Ok(FuncDef::Function(name, body))
        }
    }
}

fn collect_labels(stmt: &Stmt, env: &mut HashMap<String, String>) -> ValidationResult<()> {
    match stmt {
        Stmt::If(_, then, Some(else_)) => {
            collect_labels(then.as_ref(), env)?;
            collect_labels(else_.as_ref(), env)?;
        }
        Stmt::If(_, then, None) => {
            collect_labels(then.as_ref(), env)?;
        }
        Stmt::Labelled(label, stmt) => {
            match env.get(label) {
                Some(_) => return Err(ValidationError::DuplicateVariableDefinition(label.clone())),
                None => {
                    let new_label = make_label(label);
                    env.insert(label.clone(), new_label)
                }
            };
            collect_labels(stmt.as_ref(), env)?;
        }
        _ => {}
    }
    Ok(())
}

fn resolve_block_item(
    block_item: BlockItem,
    env: &HashMap<String, String>,
) -> ValidationResult<BlockItem> {
    Ok(match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, env)?),
        decl => decl,
    })
}

fn resolve_stmt(stmt: Stmt, env: &HashMap<String, String>) -> ValidationResult<Stmt> {
    Ok(match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(resolve_stmt(*then, env)?),
            Some(Box::new(resolve_stmt(*else_, env)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(cond, Box::new(resolve_stmt(*then, env)?), None),
        Stmt::Labelled(label, stmt) => match env.get(&label) {
            Some(label) => Stmt::Labelled(label.clone(), Box::new(resolve_stmt(*stmt, env)?)),
            None => return Err(ValidationError::UndefinedVariable(label.clone())),
        },
        Stmt::Goto(label) => match env.get(&label) {
            Some(label) => Stmt::Goto(label.clone()),
            None => return Err(ValidationError::UndefinedVariable(label)),
        },
        other => other,
    })
}
