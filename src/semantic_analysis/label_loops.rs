use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;

pub fn label_program(program: Program) -> ValidationResult<Program> {
    let program = match program {
        Program::FuncDef(func_def) => Program::FuncDef(label_func_def(func_def)?),
    };
    Ok(program)
}

fn label_func_def(func_def: FuncDef) -> ValidationResult<FuncDef> {
    let func_def = match func_def {
        FuncDef::Function(name, body) => {
            let body = body
                .into_iter()
                .map(|block_item| label_block_item(block_item, &None))
                .collect::<ValidationResult<_>>()?;

            FuncDef::Function(name, body)
        }
    };
    Ok(func_def)
}

fn label_block_item(
    block_item: BlockItem,
    current_label: &Option<String>,
) -> ValidationResult<BlockItem> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(label_stmt(stmt, current_label)?),
        decl => decl,
    };
    Ok(block_item)
}

fn label_stmt(stmt: Stmt, current_label: &Option<String>) -> ValidationResult<Stmt> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(label_stmt(*then, current_label)?),
            Some(Box::new(label_stmt(*else_, current_label)?)),
        ),
        Stmt::If(cond, then, None) => {
            Stmt::If(cond, Box::new(label_stmt(*then, current_label)?), None)
        }
        Stmt::Compound(block) => {
            let block = block
                .into_iter()
                .map(|block_item| label_block_item(block_item, current_label))
                .collect::<Result<_, _>>()?;
            Stmt::Compound(block)
        }
        Stmt::Break(_) => match current_label {
            label @ Some(_) => Stmt::Break(label.clone()),
            None => return Err(ValidationError::BreakOutsideLoop),
        },
        Stmt::Continue(_) => match current_label {
            label @ Some(_) => Stmt::Continue(label.clone()),
            None => return Err(ValidationError::ContinueOutsideLoop),
        },
        Stmt::While(cond, body, _) => {
            let label = Some(make_label("loop"));
            let body = Box::new(label_stmt(*body, &label)?);
            Stmt::While(cond, body, label)
        }
        Stmt::DoWhile(body, cond, _) => {
            let label = Some(make_label("loop"));
            let body = Box::new(label_stmt(*body, &label)?);
            Stmt::DoWhile(body, cond, label)
        }
        Stmt::For(init, cond, post, body, _) => {
            let label = Some(make_label("loop"));
            let body = Box::new(label_stmt(*body, &label)?);
            Stmt::For(init, cond, post, body, label)
        }
        other => other,
    };
    Ok(stmt)
}
