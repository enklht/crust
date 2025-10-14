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
    Ok(match func_def {
        FuncDef::Function(name, body) => {
            let mut env = HashMap::new();
            FuncDef::Function(
                name,
                body.into_iter()
                    .map(|block_item| resolve_block_item(block_item, &mut env))
                    .collect::<ValidationResult<_>>()?,
            )
        }
    })
}

fn resolve_block_item(
    block_item: BlockItem,
    env: &mut HashMap<String, String>,
) -> ValidationResult<BlockItem> {
    Ok(match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, env)?),
        BlockItem::D(decl) => BlockItem::D(resolve_decl(decl, env)?),
    })
}

fn resolve_decl(decl: Decl, env: &mut HashMap<String, String>) -> ValidationResult<Decl> {
    Ok(match decl {
        Decl::Decl(name, init) => match env.get(&name) {
            Some(_) => {
                return Err(ValidationError::DuplicateVariableDefinition(name));
            }
            None => {
                let unique_name = make_label(&name);
                env.insert(name, unique_name.clone());
                let init_resolved = init.map(|expr| resolve_expr(expr, env)).transpose()?;
                Decl::Decl(unique_name, init_resolved)
            }
        },
    })
}

fn resolve_stmt(stmt: Stmt, env: &mut HashMap<String, String>) -> ValidationResult<Stmt> {
    Ok(match stmt {
        Stmt::Return(expr) => Stmt::Return(resolve_expr(expr, env)?),
        Stmt::Expression(expr) => Stmt::Expression(resolve_expr(expr, env)?),
        Stmt::Null => Stmt::Null,
    })
}

fn resolve_expr(expr: Expr, env: &mut HashMap<String, String>) -> ValidationResult<Expr> {
    Ok(match expr {
        Expr::Assignment(left, right) => match *left {
            left @ Expr::Var(_) => Expr::Assignment(
                Box::new(resolve_expr(left, env)?),
                Box::new(resolve_expr(*right, env)?),
            ),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Expr::Var(name) => match env.get(&name) {
            Some(unique_name) => Expr::Var(unique_name.clone()),
            None => return Err(ValidationError::UndefinedVariable(name)),
        },
        Expr::Unary(op, e) => Expr::Unary(op, Box::new(resolve_expr(*e, env)?)),
        Expr::Binary(op, e1, e2) => Expr::Binary(
            op,
            Box::new(resolve_expr(*e1, env)?),
            Box::new(resolve_expr(*e2, env)?),
        ),
        other => other,
    })
}
