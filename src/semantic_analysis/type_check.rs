use super::ValidationError as Error;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::*;
use crate::models::typ::Type;

pub fn check_program(program: &mut Program) -> ValidationResult<Symbols> {
    let mut symbols = Symbols::default();
    match program {
        Program::Program(decls) => {
            decls
                .iter_mut()
                .try_for_each(|decl| check_file_scope_decl(decl, &mut symbols))?;
        }
    }
    Ok(symbols)
}

fn check_file_scope_decl(decl: &mut Decl, symbols: &mut Symbols) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, false),
        Decl::VarDecl(var_decl) => check_file_scope_var_decl(var_decl, symbols),
    }
}

fn check_local_decl(decl: &mut Decl, symbols: &mut Symbols) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, true),
        Decl::VarDecl(var_decl) => check_local_var_decl(var_decl, symbols),
    }
}

fn check_func_decl(
    func_decl: &mut FuncDecl,
    symbols: &mut Symbols,
    block_scope: bool,
) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, body, typ, storage_class) => {
            if block_scope && storage_class == &StorageClass::Static {
                return Err(Error::IncompatibleStorageClass(name.clone()));
            }

            let mut defined = false;
            let mut global = storage_class != &StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::Func {
                        typ: old_typ,
                        defined: old_defined,
                        global: old_global,
                    } if old_typ == typ => {
                        if *old_defined && body.is_some() {
                            return Err(Error::DuplicateFuncDecl(name.clone()));
                        }
                        if *old_global && storage_class == &StorageClass::Static {
                            return Err(Error::IncompatibleLinkage(name.clone()));
                        }

                        defined = *old_defined;
                        global = *old_global;
                    }
                    symbol => {
                        return Err(Error::IncompatibleFuncDecl(
                            name.clone(),
                            symbol.typ().clone(),
                            typ.clone(),
                        ));
                    }
                }
            };

            let Type::Func(param_types, ret_type) = typ.clone() else {
                unreachable!()
            };

            let symbol = Symbol::Func {
                typ: typ.clone(),
                defined: defined | body.is_some(),
                global,
            };

            symbols.insert(name.clone(), symbol);

            if let Some(body) = body {
                for (param, param_type) in params.iter().zip(param_types) {
                    symbols.insert(param.clone(), Symbol::LocalVar { typ: param_type });
                }
                check_block(body, symbols, &ret_type)?;
            }
        }
    }
    Ok(())
}

fn check_file_scope_var_decl(
    var_decl: &mut VarDecl,
    symbols: &mut Symbols,
) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, typ, storage_class) => {
            let mut init_value = match init {
                Some(init) => InitialValue::from_exp(init, typ)
                    .ok_or(Error::NonConstInitializer(name.to_string(), init.clone()))?,
                None if storage_class == &StorageClass::Extern => InitialValue::NoInitializer,
                None => InitialValue::Tentative,
            };

            let mut global = storage_class != &StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::StaticVar {
                        typ: old_typ,
                        init: old_init,
                        global: old_global,
                    } if old_typ == typ => {
                        if storage_class == &StorageClass::Extern {
                            global = *old_global;
                        } else if old_global != &global {
                            return Err(Error::IncompatibleLinkage(name.clone()));
                        }

                        match (old_init, &mut init_value) {
                            (InitialValue::Initial(_), InitialValue::Initial(_)) => {
                                return Err(Error::DuplicateVarDecl(name.clone()));
                            }
                            (old_init @ InitialValue::Initial(_), _) => {
                                init_value = old_init.clone();
                            }
                            (InitialValue::Tentative, InitialValue::Initial(_)) => {}
                            (InitialValue::Tentative, _) => init_value = InitialValue::Tentative,
                            _ => {}
                        }
                    }
                    Symbol::LocalVar { typ: old_typ } if old_typ == typ => {}
                    symbol => {
                        return Err(Error::IncompatibleVarDecl(
                            name.clone(),
                            symbol.typ().clone(),
                            typ.clone(),
                        ));
                    }
                }
            };

            let symbol = Symbol::StaticVar {
                typ: typ.clone(),
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

fn check_local_var_decl(var_decl: &mut VarDecl, symbols: &mut Symbols) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, typ, storage_class) => {
            match storage_class {
                StorageClass::Extern => {
                    if init.is_some() {
                        return Err(Error::ExternVarWithInitializer(name.clone()));
                    }
                    match symbols.get(name) {
                        Some(Symbol::Func { typ: old_typ, .. }) => {
                            return Err(Error::IncompatibleVarDecl(
                                name.clone(),
                                old_typ.clone(),
                                typ.clone(),
                            ));
                        }
                        Some(Symbol::StaticVar { typ: old_typ, .. }) if old_typ != typ => {
                            return Err(Error::IncompatibleVarDecl(
                                name.clone(),
                                old_typ.clone(),
                                typ.clone(),
                            ));
                        }
                        Some(Symbol::LocalVar { typ: old_typ, .. }) if old_typ != typ => {
                            return Err(Error::IncompatibleVarDecl(
                                name.clone(),
                                old_typ.clone(),
                                typ.clone(),
                            ));
                        }
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
                        _ => {}
                    }
                }
                StorageClass::Static => {
                    let init_value = match init {
                        Some(init) => InitialValue::from_exp(init, typ)
                            .ok_or(Error::NonConstInitializer(name.to_string(), init.clone()))?,
                        None => match typ {
                            Type::Int => InitialValue::Initial(StaticInit::Int(0)),
                            Type::Long => InitialValue::Initial(StaticInit::Long(0)),
                            _ => unreachable!(),
                        },
                    };
                    symbols.insert(
                        name.clone(),
                        Symbol::StaticVar {
                            typ: typ.clone(),
                            init: init_value,
                            global: false,
                        },
                    );
                }
                StorageClass::None => {
                    symbols.insert(name.clone(), Symbol::LocalVar { typ: typ.clone() });
                    if let Some(init) = init {
                        check_exp(init, symbols)?;
                        init.convert_to(typ);
                    }
                }
            }
            Ok(())
        }
    }
}

fn check_block(block: &mut Block, symbols: &mut Symbols, ret_type: &Type) -> ValidationResult<()> {
    block
        .iter_mut()
        .try_for_each(|block_item| check_block_item(block_item, symbols, ret_type))
}

fn check_block_item(
    block_item: &mut BlockItem,
    symbols: &mut Symbols,
    ret_type: &Type,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => check_stmt(stmt, symbols, ret_type),
        BlockItem::D(decl) => check_local_decl(decl, symbols),
    }
}

fn check_stmt(stmt: &mut Stmt, symbols: &mut Symbols, ret_type: &Type) -> ValidationResult<()> {
    match stmt {
        Stmt::Return(exp) => {
            check_exp(exp, symbols)?;
            exp.convert_to(ret_type);
        }
        Stmt::Expression(exp) => {
            check_exp(exp, symbols)?;
        }
        Stmt::If(cond, then, Some(else_)) => {
            check_exp(cond, symbols)?;
            check_stmt(then, symbols, ret_type)?;
            check_stmt(else_, symbols, ret_type)?;
        }
        Stmt::If(cond, then, None) => {
            check_exp(cond, symbols)?;
            check_stmt(then, symbols, ret_type)?;
        }
        Stmt::Labelled(_, stmt) => {
            check_stmt(stmt, symbols, ret_type)?;
        }
        Stmt::Compound(block) => {
            check_block(block, symbols, ret_type)?;
        }
        Stmt::While(cond, body, _) => {
            check_exp(cond, symbols)?;
            check_stmt(body, symbols, ret_type)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            check_stmt(body, symbols, ret_type)?;
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
            check_stmt(body, symbols, ret_type)?;
        }
        Stmt::Switch(cond, body, _, _) => {
            check_exp(cond, symbols)?;
            check_stmt(body, symbols, ret_type)?;
        }
        // Goto, Break, Continue, Null
        _ => {}
    }
    Ok(())
}

fn check_for_init(for_init: &mut ForInit, symbols: &mut Symbols) -> ValidationResult<()> {
    match for_init {
        ForInit::InitDecl(decl) => match decl {
            VarDecl::VarDecl(_, _, _, StorageClass::None) => check_local_var_decl(decl, symbols),
            VarDecl::VarDecl(name, _, _, _) => Err(Error::IncompatibleStorageClass(name.clone())),
        },
        ForInit::InitExp(exp) => check_exp(exp, symbols),
        ForInit::Null => Ok(()),
    }
}

fn check_exp(exp: &mut Exp, symbols: &mut Symbols) -> ValidationResult<()> {
    match exp {
        Exp::Const(_) => {}
        Exp::Assignment(left, right, typ) => {
            check_exp(left, symbols)?;
            check_exp(right, symbols)?;
            let left_typ = left.typ().clone();
            right.convert_to(&left_typ);
            *typ = Some(left_typ.clone());
        }
        Exp::Var(name, typ) => match symbols.get(name).unwrap() {
            Symbol::Func { .. } => {
                return Err(Error::FunctionAsVariable(name.clone()));
            }
            symbol => *typ = Some(symbol.typ().clone()),
        },
        Exp::Cast(_, inner) => {
            check_exp(inner, symbols)?;
        }
        Exp::Unary(op, inner, typ) => {
            check_exp(inner, symbols)?;
            match op {
                UnaryOp::LogicalNot => *typ = Some(Type::Int),
                UnaryOp::BitNot => {
                    let typ_inner = inner.typ().clone();
                    if inner.typ() == &Type::Double {
                        panic!()
                    }
                    *typ = Some(typ_inner)
                }
                _ => {
                    let typ_inner = inner.typ().clone();
                    *typ = Some(typ_inner)
                }
            };
        }
        Exp::Binary(op, e1, e2, typ) => {
            use BinaryOp::*;

            check_exp(e1, symbols)?;
            check_exp(e2, symbols)?;
            if matches!(op, LogicalAnd | LogicalOr) {
                *typ = Some(Type::Int);
            } else if matches!(op, LShift | RShift) {
                let left_typ = e1.typ().clone();
                *typ = Some(left_typ)
            } else {
                let common_type = e1.typ().common_type_with(e2.typ());

                e1.convert_to(&common_type);
                e2.convert_to(&common_type);
                match op {
                    Add | Subtract | Multiply | Divide | Remainder | LShift | RShift | BitAnd
                    | BitOr | BitXor => *typ = Some(common_type),
                    _ => *typ = Some(Type::Int),
                };
            }
        }
        Exp::Conditional(cond, then, else_, typ) => {
            check_exp(cond, symbols)?;
            check_exp(then, symbols)?;
            check_exp(else_, symbols)?;
            let common_type = then.typ().common_type_with(else_.typ());
            then.convert_to(&common_type);
            else_.convert_to(&common_type);
            *typ = Some(common_type);
        }
        Exp::FuncCall(name, args, typ) => {
            let Symbol::Func {
                typ: Type::Func(param_types, ret_type),
                ..
            } = symbols.get(name).unwrap()
            else {
                return Err(Error::VariableAsFunction(name.clone()));
            };
            let param_types = param_types.clone();
            let ret_type = ret_type.clone();

            if param_types.len() != args.len() {
                return Err(Error::WrongArity(
                    name.clone(),
                    param_types.len(),
                    args.len(),
                ));
            }

            for (arg, param_type) in args.iter_mut().zip(param_types) {
                check_exp(arg, symbols)?;
                arg.convert_to(&param_type);
            }

            *typ = Some(*ret_type.clone())
        }
    };
    Ok(())
}
