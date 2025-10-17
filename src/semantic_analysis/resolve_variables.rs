use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use std::collections::HashMap;

type MapEntry = (String, bool);

fn copy_env(env: &HashMap<String, MapEntry>) -> HashMap<String, MapEntry> {
    env.iter()
        .map(|(name, (unique_name, _))| (name.clone(), (unique_name.clone(), false)))
        .collect()
}

pub fn resolve_program(program: Program) -> ValidationResult<Program> {
    Ok(match program {
        Program::FuncDef(func_def) => Program::FuncDef(resolve_func_def(func_def)?),
    })
}

fn resolve_func_def(func_def: FuncDef) -> ValidationResult<FuncDef> {
    Ok(match func_def {
        FuncDef::Function(name, body) => {
            let mut env = HashMap::new();
            FuncDef::Function(name, resolve_block(body, &mut env)?)
        }
    })
}

fn resolve_block(block: Block, env: &mut HashMap<String, MapEntry>) -> ValidationResult<Block> {
    block
        .into_iter()
        .map(|block_item| resolve_block_item(block_item, env))
        .collect::<ValidationResult<_>>()
}

fn resolve_block_item(
    block_item: BlockItem,
    env: &mut HashMap<String, MapEntry>,
) -> ValidationResult<BlockItem> {
    Ok(match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, env)?),
        BlockItem::D(decl) => BlockItem::D(resolve_decl(decl, env)?),
    })
}

fn resolve_decl(decl: Decl, env: &mut HashMap<String, MapEntry>) -> ValidationResult<Decl> {
    Ok(match decl {
        Decl::Decl(name, init) => match env.get(&name) {
            Some((_, true)) => {
                return Err(ValidationError::DuplicateVariableDefinition(name));
            }
            _ => {
                let unique_name = make_label(&name);
                env.insert(name, (unique_name.clone(), true));
                let init_resolved = init.map(|exp| resolve_exp(exp, env)).transpose()?;
                Decl::Decl(unique_name, init_resolved)
            }
        },
    })
}

fn resolve_stmt(stmt: Stmt, env: &mut HashMap<String, MapEntry>) -> ValidationResult<Stmt> {
    Ok(match stmt {
        Stmt::Return(exp) => Stmt::Return(resolve_exp(exp, env)?),
        Stmt::Expression(exp) => Stmt::Expression(resolve_exp(exp, env)?),
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            resolve_exp(cond, env)?,
            Box::new(resolve_stmt(*then, env)?),
            Some(Box::new(resolve_stmt(*else_, env)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(
            resolve_exp(cond, env)?,
            Box::new(resolve_stmt(*then, env)?),
            None,
        ),
        Stmt::Labelled(label, stmt) => Stmt::Labelled(label, Box::new(resolve_stmt(*stmt, env)?)),
        Stmt::Goto(label) => Stmt::Goto(label),
        Stmt::Compound(block) => {
            let mut new_env = copy_env(env);
            Stmt::Compound(resolve_block(block, &mut new_env)?)
        }
        Stmt::Null => Stmt::Null,
        _ => todo!(),
    })
}

fn resolve_exp(exp: Exp, env: &mut HashMap<String, MapEntry>) -> ValidationResult<Exp> {
    Ok(match exp {
        Exp::Const(i) => Exp::Const(i),
        Exp::Assignment(left, right) => match *left {
            left @ Exp::Var(_) => Exp::Assignment(
                Box::new(resolve_exp(left, env)?),
                Box::new(resolve_exp(*right, env)?),
            ),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Var(name) => match env.get(&name) {
            Some((unique_name, _)) => Exp::Var(unique_name.clone()),
            None => return Err(ValidationError::UndefinedVariable(name)),
        },
        Exp::Unary(op @ (UnaryOp::PostfixIncrement | UnaryOp::PostfixDecrement), exp) => match *exp
        {
            exp @ Exp::Var(_) => Exp::Unary(op, Box::new(resolve_exp(exp, env)?)),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Unary(op, e) => Exp::Unary(op, Box::new(resolve_exp(*e, env)?)),
        Exp::Binary(op, e1, e2) => Exp::Binary(
            op,
            Box::new(resolve_exp(*e1, env)?),
            Box::new(resolve_exp(*e2, env)?),
        ),
        Exp::Conditional(cond, then, else_) => Exp::Conditional(
            Box::new(resolve_exp(*cond, env)?),
            Box::new(resolve_exp(*then, env)?),
            Box::new(resolve_exp(*else_, env)?),
        ),
    })
}
