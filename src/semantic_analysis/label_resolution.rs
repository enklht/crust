use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

pub fn resolve_program(program: Program<Plain>) -> ValidationResult<Program<Plain>> {
    let program = match program {
        Program::Program(decls) => Program::Program(
            decls
                .into_iter()
                .map(resolve_decl)
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn resolve_decl(decl: Decl<Plain>) -> ValidationResult<Decl<Plain>> {
    let decl = match decl {
        Decl::FuncDecl(func_decl) => Decl::FuncDecl(resolve_func_decl(func_decl)?),
        other => other,
    };
    Ok(decl)
}

fn resolve_func_decl(func_def: FuncDecl<Plain>) -> ValidationResult<FuncDecl<Plain>> {
    let func_def = match func_def {
        FuncDecl::FuncDecl(name, params, Some(body), typ, storage_class) => {
            let mut labels = FxHashMap::default();
            for block_item in &body {
                if let BlockItem::S(stmt) = block_item {
                    collect_labels(stmt, &mut labels)?;
                }
            }

            let body = body
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, &labels))
                .collect::<ValidationResult<_>>()?;

            FuncDecl::FuncDecl(name, params, Some(body), typ, storage_class)
        }
        other => other,
    };
    Ok(func_def)
}

fn resolve_block_item(
    block_item: BlockItem<Plain>,
    labels: &FxHashMap<String, String>,
) -> ValidationResult<BlockItem<Plain>> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, labels)?),
        decl => decl,
    };
    Ok(block_item)
}

fn collect_labels(
    stmt: &Stmt<Plain>,
    labels: &mut FxHashMap<String, String>,
) -> ValidationResult<()> {
    match stmt {
        Stmt::If(_, then, Some(else_)) => {
            collect_labels(then, labels)?;
            collect_labels(else_, labels)?;
        }
        Stmt::If(_, then, None) => collect_labels(then, labels)?,
        Stmt::Labelled(Label::Ident(label), stmt) => {
            match labels.get(label) {
                Some(_) => return Err(ValidationError::DuplicateLabel(label.clone())),
                None => {
                    let new_label = make_label(label);
                    labels.insert(label.clone(), new_label)
                }
            };
            collect_labels(stmt, labels)?;
        }
        Stmt::Labelled(_, stmt) => {
            collect_labels(stmt, labels)?;
        }
        Stmt::Compound(block) => {
            for block_item in block {
                if let BlockItem::S(stmt) = block_item {
                    collect_labels(stmt, labels)?;
                }
            }
        }
        Stmt::While(_, body, _)
        | Stmt::DoWhile(body, _, _)
        | Stmt::For(_, _, _, body, _)
        | Stmt::Switch(_, body, _, ()) => {
            collect_labels(body, labels)?;
        }
        _ => {}
    }
    Ok(())
}

fn resolve_stmt(
    stmt: Stmt<Plain>,
    labels: &FxHashMap<String, String>,
) -> ValidationResult<Stmt<Plain>> {
    let stmt = match stmt {
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            cond,
            Box::new(resolve_stmt(*then, labels)?),
            Some(Box::new(resolve_stmt(*else_, labels)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(cond, Box::new(resolve_stmt(*then, labels)?), None),
        Stmt::Labelled(Label::Ident(label), stmt) => match labels.get(&label) {
            Some(label) => Stmt::Labelled(
                Label::Ident(label.clone()),
                Box::new(resolve_stmt(*stmt, labels)?),
            ),
            None => return Err(ValidationError::UndefinedLabel(label.clone())),
        },
        Stmt::Labelled(label, stmt) => {
            Stmt::Labelled(label, Box::new(resolve_stmt(*stmt, labels)?))
        }
        Stmt::Goto(label) => match labels.get(&label) {
            Some(label) => Stmt::Goto(label.clone()),
            None => return Err(ValidationError::UndefinedLabel(label)),
        },
        Stmt::Compound(block) => {
            let block = block
                .into_iter()
                .map(|block_item| resolve_block_item(block_item, labels))
                .collect::<Result<_, _>>()?;
            Stmt::Compound(block)
        }
        Stmt::While(cond, body, label) => {
            Stmt::While(cond, Box::new(resolve_stmt(*body, labels)?), label)
        }
        Stmt::DoWhile(body, cond, label) => {
            Stmt::DoWhile(Box::new(resolve_stmt(*body, labels)?), cond, label)
        }
        Stmt::For(init, cond, post, body, label) => Stmt::For(
            init,
            cond,
            post,
            Box::new(resolve_stmt(*body, labels)?),
            label,
        ),
        Stmt::Switch(cond, body, label, ()) => {
            Stmt::Switch(cond, Box::new(resolve_stmt(*body, labels)?), label, ())
        }
        other => other,
    };
    Ok(stmt)
}
