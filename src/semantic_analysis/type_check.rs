use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::*;

pub fn check_program(program: &Program<Plain>) -> ValidationResult<Symbols> {
    let mut symbols = Symbols::default();
    match program {
        Program::Program(decls) => {
            decls
                .iter()
                .try_for_each(|decl| check_file_scope_decl(decl, &mut symbols))?;
        }
    }
    Ok(symbols)
}

fn check_file_scope_decl(decl: &Decl<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, false),
        Decl::VarDecl(var_decl) => check_file_scope_var_decl(var_decl, symbols),
    }
}

fn check_local_decl(decl: &Decl<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, true),
        Decl::VarDecl(var_decl) => check_local_var_decl(var_decl, symbols),
    }
}

fn check_func_decl(
    func_decl: &FuncDecl<Plain>,
    symbols: &mut Symbols,
    block_scope: bool,
) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, body, storage_class) => {
            if block_scope && *storage_class == StorageClass::Static {
                return Err(ValidationError::IncompatibleStorageClass(name.clone()));
            }

            let mut defined = false;
            let mut global = *storage_class != StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::Func {
                        typ: Type::Func(param_types, ret_type),
                        defined: old_defined,
                        global: old_global,
                    } if param_types.len() == params.len() => {
                        if *old_defined && body.is_some() {
                            return Err(ValidationError::DuplicateFuncDecl(name.clone()));
                        }
                        if *old_global && *storage_class == StorageClass::Static {
                            return Err(ValidationError::IncompatibleLinkage(name.clone()));
                        }

                        defined = *old_defined;
                        global = *old_global;
                    }
                    _ => return Err(ValidationError::IncompatibleFuncDecl(name.clone())),
                }
            };

            let symbol = Symbol::Func {
                // TODO: replace the place holders
                typ: Type::Func(vec![], Box::new(Type::Int)),
                defined: defined | body.is_some(),
                global,
            };

            symbols.insert(name.clone(), symbol);

            if let Some(body) = body {
                for param in params {
                    symbols.insert(param.clone(), Symbol::LocalVar { typ: Type::Int });
                }
                check_block(body, symbols)?;
            }
        }
    }
    Ok(())
}

fn check_file_scope_var_decl(var_decl: &VarDecl, symbols: &mut Symbols) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, storage_class) => {
            let mut init_value = match init {
                Some(Exp::Const(i)) => InitialValue::Initial(*i),
                None if *storage_class == StorageClass::Extern => InitialValue::NoInitializer,
                None => InitialValue::Tentative,
                _ => {
                    return Err(ValidationError::StaticVarWithNonConstInitializer(
                        name.clone(),
                    ));
                }
            };
            let mut global = *storage_class != StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::StaticVar {
                        typ,
                        init: old_init,
                        global: old_global,
                    } => {
                        if *storage_class == StorageClass::Extern {
                            global = *old_global;
                        } else if *old_global != global {
                            return Err(ValidationError::IncompatibleLinkage(name.clone()));
                        }

                        match (old_init, &mut init_value) {
                            (InitialValue::Initial(_), InitialValue::Initial(_)) => {
                                return Err(ValidationError::DuplicateVarDecl(name.clone()));
                            }
                            (InitialValue::Initial(i), _) => {
                                init_value = InitialValue::Initial(*i);
                            }
                            (InitialValue::Tentative, InitialValue::Initial(_)) => {}
                            (InitialValue::Tentative, _) => init_value = InitialValue::Tentative,
                            _ => {}
                        }
                    }
                    Symbol::LocalVar { .. } => {}
                    _ => return Err(ValidationError::IncompatibleVarDecl(name.clone())),
                }
            };

            let symbol = Symbol::StaticVar {
                typ: Type::Int,
                init: init_value,
                global,
            };

            symbols.insert(name.clone(), symbol);

            if let Some(init) = init {
                check_exp(init, symbols)?;
            }
            Ok(())
        }
    }
}

fn check_local_var_decl(var_decl: &VarDecl, symbols: &mut Symbols) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, storage_class) => {
            match storage_class {
                StorageClass::Extern => {
                    if init.is_some() {
                        return Err(ValidationError::ExternVarWithInitializer(name.clone()));
                    }
                    match symbols.get(name) {
                        Some(Symbol::Func { .. }) => {
                            return Err(ValidationError::IncompatibleVarDecl(name.clone()));
                        }
                        Some(_) => {}
                        None => {
                            symbols.insert(
                                name.clone(),
                                Symbol::StaticVar {
                                    typ: Type::Int,
                                    init: InitialValue::NoInitializer,
                                    global: true,
                                },
                            );
                        }
                    }
                }
                StorageClass::Static => {
                    let init_value = match init {
                        Some(Exp::Const(i)) => InitialValue::Initial(*i),
                        None => InitialValue::Initial(0),
                        _ => {
                            return Err(ValidationError::StaticVarWithNonConstInitializer(
                                name.clone(),
                            ));
                        }
                    };
                    symbols.insert(
                        name.clone(),
                        Symbol::StaticVar {
                            typ: Type::Int,
                            init: init_value,
                            global: false,
                        },
                    );
                }
                StorageClass::None => {
                    symbols.insert(name.clone(), Symbol::LocalVar { typ: Type::Int });
                    if let Some(init) = init {
                        check_exp(init, symbols)?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn check_block(block: &Block<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    block
        .iter()
        .try_for_each(|block_item| check_block_item(block_item, symbols))
}

fn check_block_item(block_item: &BlockItem<Plain>, symbols: &mut Symbols) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => check_stmt(stmt, symbols),
        BlockItem::D(decl) => check_local_decl(decl, symbols),
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
        ForInit::InitDecl(decl) => match decl {
            VarDecl::VarDecl(_, _, StorageClass::None) => check_local_var_decl(decl, symbols),
            VarDecl::VarDecl(name, _, _) => {
                Err(ValidationError::IncompatibleStorageClass(name.clone()))
            }
        },
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
            Some(Symbol::Func { .. }) => {
                return Err(ValidationError::FunctionAsVariable(name.clone()));
            }
            Some(_) => {}
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
            Some(Symbol::Func {
                typ: Type::Func(params, _),
                ..
            }) if params.len() == args.len() => {
                args.iter().try_for_each(|arg| check_exp(arg, symbols))?;
            }
            Some(Symbol::Func {
                typ: Type::Func(params, _),
                ..
            }) => {
                return Err(ValidationError::WrongArity(
                    name.clone(),
                    params.len(),
                    args.len(),
                ));
            }
            Some(_) => return Err(ValidationError::VariableAsFunction(name.clone())),
            _ => unreachable!(),
        },
    };
    Ok(())
}
