use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;

type Ident = &'static str;

pub fn label_program(program: &mut Program) -> ValidationResult<()> {
    match program {
        Program::Program(func_decls) => func_decls.iter_mut().try_for_each(label_decl),
    }
}

fn label_decl(decl: &mut Decl) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => label_func_decl(func_decl),
        Decl::VarDecl(_) => Ok(()),
    }
}

fn label_func_decl(func_decl: &mut FuncDecl) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(_, _, Some(body), _, _) => body
            .iter_mut()
            .try_for_each(|block_item| label_block_item(block_item, None, None)),
        _ => Ok(()),
    }
}

fn label_block_item(
    block_item: &mut BlockItem,
    break_label: Option<Ident>,
    continue_label: Option<Ident>,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => label_stmt(stmt, break_label, continue_label),
        BlockItem::D(decl) => label_decl(decl),
    }
}

fn label_stmt(
    stmt: &mut Stmt,
    break_label: Option<Ident>,
    continue_label: Option<Ident>,
) -> ValidationResult<()> {
    match stmt {
        Stmt::If(_, then, else_) => {
            label_stmt(then, break_label, continue_label)?;
            if let Some(else_) = else_ {
                label_stmt(else_, break_label, continue_label)?;
            }
        }
        Stmt::Labelled(_, stmt) => {
            label_stmt(stmt, break_label, continue_label)?;
        }
        Stmt::Compound(block) => block
            .iter_mut()
            .try_for_each(|block_item| label_block_item(block_item, break_label, continue_label))?,
        Stmt::Break(label) => match break_label {
            Some(_) => *label = break_label,
            None => return Err(ValidationError::BreakOutsideLoop),
        },
        Stmt::Continue(label) => match continue_label {
            Some(_) => *label = continue_label,
            None => return Err(ValidationError::ContinueOutsideLoop),
        },
        Stmt::While(_, body, label)
        | Stmt::DoWhile(body, _, label)
        | Stmt::For(_, _, _, body, label) => {
            let new_label = make_label("loop");
            label_stmt(body, Some(new_label), Some(new_label))?;
            *label = Some(new_label);
        }
        Stmt::Switch(_, body, label, _) => {
            let new_label = make_label("switch");
            label_stmt(body, Some(new_label), continue_label)?;
            *label = Some(new_label);
        }
        // Return, Goto, Expression, Null
        _ => {}
    }
    Ok(())
}
