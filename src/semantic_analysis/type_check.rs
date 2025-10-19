use super::Symbols;
use super::Type;
use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;

pub fn check_program(program: &Program<Plain>) -> ValidationResult<Symbols> {
    let mut symbols = Symbols::default();
    match program {
        Program::Program(decls) => {
            decls
                .iter()
                .try_for_each(|decl| check_decl(decl, &mut symbols))?;
        }
    }
    Ok(symbols)
}

fn check_block(block: &Block<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    block
        .iter()
        .try_for_each(|block_item| check_block_item(block_item, symbols))
}

fn check_block_item(block_item: &BlockItem<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => check_stmt(stmt, symbols),
        BlockItem::D(decl) => check_decl(decl, symbols),
    }
}

fn check_decl(decl: &Decl<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols),
        Decl::VarDecl(var_decl) => check_var_decl(var_decl, symbols),
    }
}

fn check_func_decl(func_decl: &FuncDecl<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, body, storage_class) => {
            let type_ = Type::Fn(params.len());
            let mut defined = false;

            if let Some((type_old, defined_old)) = symbols.get(name) {
                if *type_old != type_ {
                    return Err(ValidationError::IncompatibleFuncDecl(
                        name.clone(),
                        type_old.clone(),
                        type_,
                    ));
                }
                if *defined_old && body.is_some() {
                    return Err(ValidationError::DuplicateFuncDecl(name.clone()));
                }
                defined = *defined_old
            }

            symbols.insert(name.clone(), (type_, defined || body.is_some()));

            if let Some(body) = body {
                for param in params {
                    symbols.insert(param.clone(), (Type::Int, defined));
                }
                check_block(body, symbols)?;
            }
        }
    }
    Ok(())
}

fn check_var_decl(var_decl: &VarDecl, symbols: &mut Symbols) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, storage_class) => {
            symbols.insert(name.clone(), (Type::Int, true));
            if let Some(init) = init {
                check_exp(init, symbols)?;
            }
            Ok(())
        }
    }
}

fn check_stmt(stmt: &Stmt<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match stmt {
        Stmt::Return(exp) => {
            check_exp(exp, symbols)?;
        }
        Stmt::Expression(exp) => {
            check_exp(exp, symbols)?;
        }
        Stmt::If(cond, then, Some(else_)) => {
            check_exp(cond, symbols)?;
            check_stmt(then, symbols)?;
            check_stmt(else_, symbols)?;
        }
        Stmt::If(cond, then, None) => {
            check_exp(cond, symbols)?;
            check_stmt(then, symbols)?;
        }
        Stmt::Labelled(_, stmt) => {
            check_stmt(stmt, symbols)?;
        }
        Stmt::Compound(block) => {
            check_block(block, symbols)?;
        }
        Stmt::While(cond, body, _) => {
            check_exp(cond, symbols)?;
            check_stmt(body, symbols)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            check_stmt(body, symbols)?;
            check_exp(cond, symbols)?;
        }
        Stmt::For(init, cond, post, body, _) => {
            check_for_init(init, symbols)?;
            if let Some(cond) = cond {
                check_exp(cond, symbols)?;
            }
            if let Some(post) = post {
                check_exp(post, symbols)?;
            }
            check_stmt(body, symbols)?;
        }
        Stmt::Switch(cond, body, _, _) => {
            check_exp(cond, symbols)?;
            check_stmt(body, symbols)?;
        }
        // Goto, Break, Continue, Null
        _ => {}
    }
    Ok(())
}

fn check_for_init(for_init: &ForInit, symbols: &mut Symbols) -> ValidationResult<()> {
    match for_init {
        ForInit::InitDecl(decl) => check_var_decl(decl, symbols),
        ForInit::InitExp(exp) => check_exp(exp, symbols),
        ForInit::Null => Ok(()),
    }
}

fn check_exp(exp: &Exp, symbols: &mut Symbols) -> ValidationResult<()> {
    match exp {
        Exp::Const(_) => {}
        Exp::Assignment(left, right) => {
            check_exp(left, symbols)?;
            check_exp(right, symbols)?;
        }
        Exp::Var(name) => match symbols.get(name) {
            Some(&(Type::Int, _)) => {}
            Some(_) => return Err(ValidationError::FunctionAsVariable(name.clone())),
            None => {}
        },
        Exp::Unary(_, exp) => {
            check_exp(exp, symbols)?;
        }
        Exp::Binary(_, e1, e2) => {
            check_exp(e1, symbols)?;
            check_exp(e2, symbols)?;
        }
        Exp::Conditional(cond, then, else_) => {
            check_exp(cond, symbols)?;
            check_exp(then, symbols)?;
            check_exp(else_, symbols)?;
        }
        Exp::FuncCall(name, args) => match symbols.get(name) {
            Some((Type::Int, _)) => return Err(ValidationError::VariableAsFunction(name.clone())),
            Some((Type::Fn(param_count), _)) if *param_count == args.len() => {
                args.iter().try_for_each(|arg| check_exp(arg, symbols))?;
            }
            Some((Type::Fn(param_count), _)) => {
                return Err(ValidationError::WrongArity(
                    name.clone(),
                    *param_count,
                    args.len(),
                ));
            }
            _ => unreachable!(),
        },
    };
    Ok(())
}
