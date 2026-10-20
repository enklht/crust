use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::typ::Type;
use crate::utils::Ident;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

#[derive(Debug)]
struct Context {
    cases: FxHashMap<Const, Ident>,
    default: Option<Ident>,
    typ: Type,
}

pub fn resolve_program(program: &mut Program) -> ValidationResult<()> {
    match program {
        Program::Program(func_decls) => func_decls.iter_mut().try_for_each(resolve_decl)?,
    };
    Ok(())
}

fn resolve_decl(decl: &mut Decl) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => resolve_func_decl(func_decl)?,
        Decl::VarDecl(_) => {}
    };
    Ok(())
}

fn resolve_func_decl(func_def: &mut FuncDecl) -> ValidationResult<()> {
    if let FuncDecl::FuncDecl(_, _, Some(body), _, _) = func_def {
        body.iter_mut()
            .try_for_each(|block_item| resolve_block_item(block_item, &mut None))
    } else {
        Ok(())
    }
}

fn resolve_block_item(
    block_item: &mut BlockItem,
    context: &mut Option<&mut Context>,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => resolve_stmt(stmt, context)?,
        BlockItem::D(decl) => resolve_decl(decl)?,
    }
    Ok(())
}

fn resolve_stmt(stmt: &mut Stmt, context: &mut Option<&mut Context>) -> ValidationResult<()> {
    match stmt {
        Stmt::If(_, then, else_) => {
            resolve_stmt(then, context)?;
            if let Some(else_) = else_ {
                resolve_stmt(else_, context)?;
            }
        }
        Stmt::Labelled(label @ Label::Case(_), stmt) => {
            let new_label = make_label("case");
            let Label::Case(exp) = std::mem::replace(label, Label::Ident(new_label)) else {
                unreachable!()
            };

            if let Some(context) = context.as_mut() {
                let ExpKind::Const(c) = exp.kind else {
                    return Err(ValidationError::Switch(format!(
                        "Case value must be a constant expression, found: {exp:?}"
                    )));
                };
                if c.typ() == &Type::Double {
                    panic!()
                }

                let c = c.convert_to(&context.typ);
                if context.cases.contains_key(&c) {
                    return Err(ValidationError::Switch(format!(
                        "Duplicate case value {c:?} in switch statement"
                    )));
                }
                context.cases.insert(c, new_label);
            } else {
                return Err(ValidationError::Switch(
                    "'case' statement can only appear inside a switch".to_string(),
                ));
            }

            resolve_stmt(stmt, context)?;
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

            resolve_stmt(stmt, context)?;
        }
        Stmt::Compound(block) => {
            block
                .iter_mut()
                .try_for_each(|block_item| resolve_block_item(block_item, context))?;
        }
        Stmt::Labelled(_, body)
        | Stmt::While(_, body, _)
        | Stmt::DoWhile(body, _, _)
        | Stmt::For(_, _, _, body, _) => resolve_stmt(body, context)?,
        Stmt::Switch(cond, body, _, info) => {
            let cond_type = cond.typ();
            if cond_type == &Type::Double {
                panic!()
            }

            let mut new_context = Context {
                cases: FxHashMap::default(),
                default: None,
                typ: cond_type.clone(),
            };
            resolve_stmt(body, &mut Some(&mut new_context))?;
            *info = Some((new_context.cases, new_context.default));
        }
        // Return, Goto, Expression, Break, Continue, Null
        _ => {}
    };
    Ok(())
}
