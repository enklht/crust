use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::*;
use rustc_hash::FxHashMap;

type Types = FxHashMap<Exp, Type>;

pub fn check_program(program: &mut Program<Plain>) -> ValidationResult<(Symbols, Types)> {
    let mut symbols = Symbols::default();
    let mut types: Types = FxHashMap::default();
    match program {
        Program::Program(decls) => {
            decls
                .iter_mut()
                .try_for_each(|decl| check_file_scope_decl(decl, &mut symbols, &mut types))?;
        }
    }
    Ok((symbols, types))
}

fn check_file_scope_decl(
    decl: &mut Decl<Plain>,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, false, types),
        Decl::VarDecl(var_decl) => check_file_scope_var_decl(var_decl, symbols, types),
    }
}

fn check_local_decl(
    decl: &mut Decl<Plain>,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, true, types),
        Decl::VarDecl(var_decl) => check_local_var_decl(var_decl, symbols, types),
    }
}

fn check_func_decl(
    func_decl: &mut FuncDecl<Plain>,
    symbols: &mut Symbols,
    block_scope: bool,
    types: &mut Types,
) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, body, typ, storage_class) => {
            if block_scope && *storage_class == StorageClass::Static {
                return Err(ValidationError::IncompatibleStorageClass(name.clone()));
            }

            let mut defined = false;
            let mut global = *storage_class != StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::Func {
                        typ: old_typ,
                        defined: old_defined,
                        global: old_global,
                    } if old_typ == typ => {
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

            let Type::Func(param_types, _) = typ.clone() else {
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
                check_block(body, symbols, types)?;
            }
        }
    }
    Ok(())
}

fn check_file_scope_var_decl(
    var_decl: &mut VarDecl,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, typ, storage_class) => {
            let mut init_value = match init {
                Some(Exp::Const(c)) => match c {
                    Const::Int(i) => InitialValue::Initial(StaticInit::IntInit(*i)),
                    Const::Long(i) => InitialValue::Initial(StaticInit::LongInit(*i)),
                },
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
                        typ: old_typ,
                        init: old_init,
                        global: old_global,
                    } if old_typ == typ => {
                        if *storage_class == StorageClass::Extern {
                            global = *old_global;
                        } else if *old_global != global {
                            return Err(ValidationError::IncompatibleLinkage(name.clone()));
                        }

                        match (old_init, &mut init_value) {
                            (InitialValue::Initial(_), InitialValue::Initial(_)) => {
                                return Err(ValidationError::DuplicateVarDecl(name.clone()));
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
                    _ => return Err(ValidationError::IncompatibleVarDecl(name.clone())),
                }
            };

            let symbol = Symbol::StaticVar {
                typ: typ.clone(),
                init: init_value,
                global,
            };

            symbols.insert(name.clone(), symbol);

            if let Some(init) = init {
                check_exp(init, symbols, types)?;
            }
            Ok(())
        }
    }
}

fn check_local_var_decl(
    var_decl: &mut VarDecl,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, typ, storage_class) => {
            match storage_class {
                StorageClass::Extern => {
                    if init.is_some() {
                        return Err(ValidationError::ExternVarWithInitializer(name.clone()));
                    }
                    println!("{:?}", symbols.get(name));
                    match symbols.get(name) {
                        Some(Symbol::Func { .. }) => {
                            return Err(ValidationError::IncompatibleVarDecl(name.clone()));
                        }
                        Some(Symbol::StaticVar { typ: old_typ, .. }) if old_typ != typ => {
                            return Err(ValidationError::IncompatibleVarDecl(name.clone()));
                        }
                        Some(Symbol::LocalVar { typ: old_typ, .. }) if old_typ != typ => {
                            return Err(ValidationError::IncompatibleVarDecl(name.clone()));
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
                        Some(Exp::Const(c)) => match c {
                            Const::Int(i) => InitialValue::Initial(StaticInit::IntInit(*i)),
                            Const::Long(i) => InitialValue::Initial(StaticInit::LongInit(*i)),
                        },
                        None => InitialValue::Initial(StaticInit::IntInit(0)),
                        _ => {
                            return Err(ValidationError::StaticVarWithNonConstInitializer(
                                name.clone(),
                            ));
                        }
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
                        check_exp(init, symbols, types)?;
                        let converted_init = convert_to(init.clone(), typ, types);
                        types.insert(converted_init.clone(), typ.clone());
                        *init = converted_init;
                    }
                }
            }
            Ok(())
        }
    }
}

fn check_block(
    block: &mut Block<Plain>,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    block
        .iter_mut()
        .try_for_each(|block_item| check_block_item(block_item, symbols, types))
}

fn check_block_item(
    block_item: &mut BlockItem<Plain>,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => check_stmt(stmt, symbols, types),
        BlockItem::D(decl) => check_local_decl(decl, symbols, types),
    }
}

fn check_stmt(
    stmt: &mut Stmt<Plain>,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match stmt {
        Stmt::Return(exp) => {
            check_exp(exp, symbols, types)?;
        }
        Stmt::Expression(exp) => {
            check_exp(exp, symbols, types)?;
        }
        Stmt::If(cond, then, Some(else_)) => {
            check_exp(cond, symbols, types)?;
            check_stmt(then, symbols, types)?;
            check_stmt(else_, symbols, types)?;
        }
        Stmt::If(cond, then, None) => {
            check_exp(cond, symbols, types)?;
            check_stmt(then, symbols, types)?;
        }
        Stmt::Labelled(_, stmt) => {
            check_stmt(stmt, symbols, types)?;
        }
        Stmt::Compound(block) => {
            check_block(block, symbols, types)?;
        }
        Stmt::While(cond, body, _) => {
            check_exp(cond, symbols, types)?;
            check_stmt(body, symbols, types)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            check_stmt(body, symbols, types)?;
            check_exp(cond, symbols, types)?;
        }
        Stmt::For(init, cond, post, body, _) => {
            check_for_init(init, symbols, types)?;
            if let Some(cond) = cond {
                check_exp(cond, symbols, types)?;
            }
            if let Some(post) = post {
                check_exp(post, symbols, types)?;
            }
            check_stmt(body, symbols, types)?;
        }
        Stmt::Switch(cond, body, _, _) => {
            check_exp(cond, symbols, types)?;
            check_stmt(body, symbols, types)?;
        }
        // Goto, Break, Continue, Null
        _ => {}
    }
    Ok(())
}

fn check_for_init(
    for_init: &mut ForInit,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match for_init {
        ForInit::InitDecl(decl) => match decl {
            VarDecl::VarDecl(_, _, _, StorageClass::None) => {
                check_local_var_decl(decl, symbols, types)
            }
            VarDecl::VarDecl(name, _, _, _) => {
                Err(ValidationError::IncompatibleStorageClass(name.clone()))
            }
        },
        ForInit::InitExp(exp) => check_exp(exp, symbols, types),
        ForInit::Null => Ok(()),
    }
}

fn check_exp(exp: &mut Exp, symbols: &mut Symbols, types: &mut Types) -> ValidationResult<()> {
    match exp {
        Exp::Const(c) => {
            match c {
                Const::Int(_) => types.insert(exp.clone(), Type::Int),
                Const::Long(_) => types.insert(exp.clone(), Type::Long),
            };
        }
        Exp::Assignment(left, right) => {
            check_exp(left, symbols, types)?;
            check_exp(right, symbols, types)?;
            let left_typ = types.get(left).unwrap().clone();
            let converted_right = convert_to(*right.clone(), &left_typ, types);
            let new_exp = Exp::Assignment(left.clone(), Box::new(converted_right));
            types.insert(new_exp.clone(), left_typ);
            *exp = new_exp;
        }
        Exp::Var(name) => match symbols.get(name).unwrap() {
            Symbol::Func { .. } => {
                return Err(ValidationError::FunctionAsVariable(name.clone()));
            }
            symbol => {
                types.insert(exp.clone(), symbol.typ());
            }
        },
        Exp::Cast(typ, inner) => {
            check_exp(inner, symbols, types)?;
            let typ = typ.clone();
            types.insert(exp.clone(), typ);
        }
        Exp::Unary(op, inner) => {
            check_exp(inner, symbols, types)?;
            match op {
                UnaryOp::Not => types.insert(exp.clone(), Type::Int),
                _ => {
                    let typ = types.get(inner).unwrap().clone();
                    types.insert(exp.clone(), typ)
                }
            };
        }
        Exp::Binary(op, e1, e2) => {
            use BinaryOp::*;

            check_exp(e1, symbols, types)?;
            check_exp(e2, symbols, types)?;
            if matches!(op, And | Or) {
                types.insert(exp.clone(), Type::Int);
                return Ok(());
            } else if matches!(op, LShift | RShift) {
                let left_typ = types.get(e1).unwrap().clone();
                types.insert(exp.clone(), left_typ);
                return Ok(());
            }
            let common_type = get_common_type(types.get(e1).unwrap(), types.get(e2).unwrap());

            let e1 = convert_to(*e1.clone(), &common_type, types);
            let e2 = convert_to(*e2.clone(), &common_type, types);
            let new_exp = Exp::Binary(op.clone(), Box::new(e1), Box::new(e2));
            match op {
                Add | Subtract | Multiply | Divide | Remainder | LShift | RShift | BitAnd
                | BitOr => types.insert(new_exp.clone(), common_type),
                _ => types.insert(new_exp.clone(), Type::Int),
            };
            *exp = new_exp;
        }
        Exp::Conditional(cond, then, else_) => {
            check_exp(cond, symbols, types)?;
            check_exp(then, symbols, types)?;
            check_exp(else_, symbols, types)?;
            types.insert(exp.clone(), Type::Int);
        }
        Exp::FuncCall(name, args) => {
            let Symbol::Func {
                typ: Type::Func(param_types, ret_type),
                ..
            } = symbols.get(name).unwrap()
            else {
                return Err(ValidationError::VariableAsFunction(name.clone()));
            };
            let param_types = param_types.clone();
            let ret_type = ret_type.clone();

            if param_types.len() != args.len() {
                return Err(ValidationError::WrongArity(
                    name.clone(),
                    param_types.len(),
                    args.len(),
                ));
            }

            let mut new_args = Vec::new();
            for (arg, param_type) in args.iter_mut().zip(param_types) {
                check_exp(arg, symbols, types)?;
                new_args.push(convert_to(arg.clone(), &param_type, types))
            }

            let new_exp = Exp::FuncCall(name.clone(), new_args);
            types.insert(new_exp.clone(), *ret_type);
            *exp = new_exp;
        }
    };
    Ok(())
}

fn get_common_type(typ1: &Type, typ2: &Type) -> Type {
    if typ1 == typ2 {
        typ1.clone()
    } else {
        Type::Long
    }
}

fn convert_to(exp: Exp, typ: &Type, types: &mut Types) -> Exp {
    if types.get(&exp).unwrap() == typ {
        exp
    } else {
        let new_exp = Exp::Cast(typ.clone(), Box::new(exp));
        types.insert(new_exp.clone(), typ.clone());
        new_exp
    }
}
