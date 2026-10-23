use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::typ::{TypeId, Types};
use crate::utils::Ident;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

#[derive(Debug)]
struct Context {
    cases: FxHashMap<Const, Ident>,
    default: Option<Ident>,
    typ: TypeId,
}

pub fn resolve_program(program: &mut Program, types: &Types) -> ValidationResult<()> {
    match program {
        Program::Program(func_decls) => func_decls
            .iter_mut()
            .try_for_each(|decl| resolve_decl(decl, types))?,
    };
    Ok(())
}

fn resolve_decl(decl: &mut Decl, types: &Types) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => resolve_func_decl(func_decl, types)?,
        Decl::VarDecl(_) => {}
    };
    Ok(())
}

fn resolve_func_decl(func_def: &mut FuncDecl, types: &Types) -> ValidationResult<()> {
    if let FuncDecl::FuncDecl(_, _, Some(body), _, _) = func_def {
        body.iter_mut()
            .try_for_each(|block_item| resolve_block_item(block_item, &mut None, types))
    } else {
        Ok(())
    }
}

fn resolve_block_item(
    block_item: &mut BlockItem,
    context: &mut Option<&mut Context>,
    types: &Types,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => resolve_stmt(stmt, context, types)?,
        BlockItem::D(decl) => resolve_decl(decl, types)?,
    }
    Ok(())
}

fn resolve_stmt(
    stmt: &mut Stmt,
    context: &mut Option<&mut Context>,
    types: &Types,
) -> ValidationResult<()> {
    match stmt {
        Stmt::If(_, then, else_) => {
            resolve_stmt(then, context, types)?;
            if let Some(else_) = else_ {
                resolve_stmt(else_, context, types)?;
            }
        }
        Stmt::Labelled(label @ Label::Case(_), stmt) => {
            let new_label = make_label("case");
            let Label::Case(exp) = std::mem::replace(label, Label::Ident(new_label)) else {
                unreachable!()
            };

            if let Some(context) = context.as_mut() {
                let ExpKind::Const(c) = exp.kind else {
                    return Err(ValidationError::Switch(
                        "Case value must be an integer constant".to_string(),
                    ));
                };
                if c.typ(types) == types.double {
                    return Err(ValidationError::Switch(
                        "Case value must be an integer constant, found 'double'".to_string(),
                    ));
                }

                let c = c.convert_to(context.typ, types);
                if context.cases.contains_key(&c) {
                    return Err(ValidationError::Switch(format!(
                        "Duplicate case value {} in switch statement",
                        match c {
                            Const::Int(value) => value.to_string(),
                            Const::Long(value) => value.to_string(),
                            Const::UInt(value) => value.to_string(),
                            Const::ULong(value) => value.to_string(),
                            Const::Double(_) => unreachable!(),
                        }
                    )));
                }
                context.cases.insert(c, new_label);
            } else {
                return Err(ValidationError::Switch(
                    "'case' statement can only appear inside a switch".to_string(),
                ));
            }

            resolve_stmt(stmt, context, types)?;
        }
        Stmt::Labelled(label @ Label::Default, stmt) => {
            let new_label = make_label("default");
            *label = Label::Ident(new_label);

            if let Some(context) = context.as_mut() {
                match context.default {
                    Some(_) => {
                        return Err(ValidationError::Switch(
                            "Switch statement cannot have multiple 'default' cases".to_string(),
                        ));
                    }
                    None => context.default = Some(new_label),
                }
            } else {
                return Err(ValidationError::Switch(
                    "'default' statement can only appear inside a switch".to_string(),
                ));
            }

            resolve_stmt(stmt, context, types)?;
        }
        Stmt::Compound(block) => {
            block
                .iter_mut()
                .try_for_each(|block_item| resolve_block_item(block_item, context, types))?;
        }
        Stmt::Labelled(_, body)
        | Stmt::While(_, body, _)
        | Stmt::DoWhile(body, _, _)
        | Stmt::For(_, _, _, body, _) => resolve_stmt(body, context, types)?,
        Stmt::Switch(cond, body, _, info) => {
            let cond_type = cond.typ();
            if cond_type == types.double {
                panic!()
            }

            let mut new_context = Context {
                cases: FxHashMap::default(),
                default: None,
                typ: cond_type,
            };
            resolve_stmt(body, &mut Some(&mut new_context), types)?;
            *info = Some((new_context.cases, new_context.default));
        }
        // Return, Goto, Expression, Break, Continue, Null
        _ => {}
    };
    Ok(())
}
