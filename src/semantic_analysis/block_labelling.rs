use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;

pub fn label_program(program: Program<Plain>) -> ValidationResult<Program<Labelled>> {
    let program = match program {
        Program::Program(func_decls) => Program::Program(
            func_decls
                .into_iter()
                .map(label_func_decl)
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn label_func_decl(func_decl: FuncDecl<Plain>) -> ValidationResult<FuncDecl<Labelled>> {
    let func_decl = match func_decl {
        FuncDecl::FuncDecl(name, params, Some(body)) => {
            let body = body
                .into_iter()
                .map(|block_item| label_block_item(block_item, None, None))
                .collect::<ValidationResult<_>>()?;

            FuncDecl::FuncDecl(name, params, Some(body))
        }
        FuncDecl::FuncDecl(name, params, None) => FuncDecl::FuncDecl(name, params, None),
    };
    Ok(func_decl)
}

fn label_block_item(
    block_item: BlockItem<Plain>,
    break_label: Option<&String>,
    continue_label: Option<&String>,
) -> ValidationResult<BlockItem<Labelled>> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(label_stmt(stmt, break_label, continue_label)?),
        BlockItem::D(decl) => BlockItem::D(convert_decl(decl)),
    };
    Ok(block_item)
}

fn convert_decl(decl: Decl<Plain>) -> Decl<Labelled> {
    match decl {
        Decl::VarDecl(var_decl) => Decl::VarDecl(var_decl),
        Decl::FuncDecl(func_decl) => {
            let func_decl = match func_decl {
                FuncDecl::FuncDecl(name, params, None) => FuncDecl::FuncDecl(name, params, None),
                _ => unreachable!(),
            };
            Decl::FuncDecl(func_decl)
        }
    }
}

fn label_stmt(
    stmt: Stmt<Plain>,
    break_label: Option<&String>,
    continue_label: Option<&String>,
) -> ValidationResult<Stmt<Labelled>> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(label_stmt(*then, break_label, continue_label)?),
            Some(Box::new(label_stmt(*else_, break_label, continue_label)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(
            cond,
            Box::new(label_stmt(*then, break_label, continue_label)?),
            None,
        ),
        Stmt::Labelled(label, stmt) => Stmt::Labelled(
            label,
            Box::new(label_stmt(*stmt, break_label, continue_label)?),
        ),
        Stmt::Compound(block) => {
            let block = block
                .into_iter()
                .map(|block_item| label_block_item(block_item, break_label, continue_label))
                .collect::<Result<_, _>>()?;
            Stmt::Compound(block)
        }
        Stmt::Break(_) => match break_label.cloned() {
            Some(label) => Stmt::Break(label),
            None => return Err(ValidationError::BreakOutsideLoop),
        },
        Stmt::Continue(_) => match continue_label {
            Some(label) => Stmt::Continue(label.clone()),
            None => return Err(ValidationError::ContinueOutsideLoop),
        },
        Stmt::While(cond, body, _) => {
            let label = make_label("loop");
            let body = Box::new(label_stmt(*body, Some(&label), Some(&label))?);
            Stmt::While(cond, body, label)
        }
        Stmt::DoWhile(body, cond, _) => {
            let label = make_label("loop");
            let body = Box::new(label_stmt(*body, Some(&label), Some(&label))?);
            Stmt::DoWhile(body, cond, label)
        }
        Stmt::For(init, cond, post, body, _) => {
            let label = make_label("loop");
            let body = Box::new(label_stmt(*body, Some(&label), Some(&label))?);
            Stmt::For(init, cond, post, body, label)
        }
        Stmt::Switch(cond, body, _, ()) => {
            let label = make_label("switch");
            let body = Box::new(label_stmt(*body, Some(&label), continue_label)?);
            Stmt::Switch(cond, body, label, ())
        }
        Stmt::Return(exp) => Stmt::Return(exp),
        Stmt::Goto(label) => Stmt::Goto(label),
        Stmt::Expression(exp) => Stmt::Expression(exp),
        Stmt::Null => Stmt::Null,
    };
    Ok(stmt)
}
