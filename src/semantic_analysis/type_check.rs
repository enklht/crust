use super::ValidationError as Error;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::*;
use crate::models::typ::Type;
use crate::utils::make_temporary;
use std::cmp::Ordering;

pub fn common_type_with(left: &Exp, right: &Exp) -> ValidationResult<Type> {
    let left_type = left.typ();
    let right_type = right.typ();

    if left_type == right_type {
        Ok(left_type.clone())
    } else if left_type == &Type::Double || right_type == &Type::Double {
        Ok(Type::Double)
    } else {
        match left_type
            .size()
            .ok_or(Error::Type(format!("{left_type} has no size")))?
            .cmp(
                &right_type
                    .size()
                    .ok_or(Error::Type(format!("{right_type} has no size")))?,
            ) {
            Ordering::Equal if left_type.is_signed() => Ok(right_type.clone()),
            Ordering::Equal => Ok(left_type.clone()),
            Ordering::Greater => Ok(left_type.clone()),
            Ordering::Less => Ok(right_type.clone()),
        }
    }
}

pub fn common_pointer_type_with(left: &Exp, right: &Exp) -> ValidationResult<Type> {
    let left_type = left.typ();
    let right_type = right.typ();

    if !left_type.is_pointer() && !right_type.is_pointer() {
        Err(Error::Type(format!(
            "{left_type} or {right_type} must be pointer"
        )))
    } else if left_type == right_type {
        Ok(left_type.clone())
    } else if left.is_null_pointer_constant() {
        Ok(right_type.clone())
    } else if right.is_null_pointer_constant() {
        Ok(left_type.clone())
    } else {
        Err(Error::Type(format!(
            "{left_type} and {right_type} has no common pointer type"
        )))
    }
}

pub fn convert_to(exp: &mut Exp, typ: &Type) {
    if exp.typ() != typ {
        let inner = std::mem::replace(exp, Exp::new(ExpKind::Const(Const::Int(0))));
        *exp = Exp {
            kind: ExpKind::Cast(typ.clone(), Box::new(inner)),
            typ: Some(typ.clone()),
        };
    }
}

pub fn convert_by_assignment_to(exp: &mut Exp, target_type: &Type) -> ValidationResult<()> {
    match exp.typ() {
        typ if typ == target_type => {}
        typ if typ.is_arithmetic() && target_type.is_arithmetic() => convert_to(exp, target_type),
        _ if exp.is_null_pointer_constant() && target_type.is_pointer() => {
            convert_to(exp, target_type)
        }
        _ => {
            return Err(Error::Type(format!(
                "'{exp:?}' cannot converted to '{target_type}' by assignment"
            )));
        }
    }
    Ok(())
}

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

fn check_type(typ: &Type) -> ValidationResult<()> {
    match typ {
        Type::Array(element_type, _) => {
            check_type(element_type)?;
            if !typ.size().is_some_and(|size| size <= isize::MAX as usize) {
                return Err(Error::Type(
                    "Array size exceeds the maximum supported object size".to_string(),
                ));
            }
        }
        Type::Pointer(referenced_type) => check_type(referenced_type)?,
        Type::Func(param_types, ret_type) => {
            for param_type in param_types {
                check_type(param_type)?;
            }
            check_type(ret_type)?;
        }
        _ => {}
    }
    Ok(())
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
            check_type(typ)?;
            let Type::Func(param_types, ret_type) = typ.clone() else {
                unreachable!()
            };

            if matches!(*ret_type, Type::Array(_, _)) {
                return Err(Error::Type(format!("A function cannot return an array")));
            }

            let param_types: Vec<Type> = param_types
                .iter()
                .map(|t| match t {
                    Type::Array(element_type, _) => Type::Pointer(element_type.clone()),
                    _ => t.clone(),
                })
                .collect();

            *typ = Type::Func(param_types.clone(), ret_type.clone());

            if block_scope && storage_class == &StorageClass::Static {
                return Err(Error::FuncDecl(format!(
                    "Incompatible storage class for function '{name}'"
                )));
            }

            let mut defined = false;
            let mut global = storage_class != &StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::Func {
                        typ: old_type,
                        defined: old_defined,
                        global: old_global,
                    } if old_type == typ => {
                        if *old_defined && body.is_some() {
                            return Err(Error::FuncDecl(format!(
                                "Function '{name}' has already been declared",
                            )));
                        }
                        if *old_global && storage_class == &StorageClass::Static {
                            return Err(Error::FuncDecl(format!(
                                "Incompatible linkage for function '{name}'"
                            )));
                        }

                        defined = *old_defined;
                        global = *old_global;
                    }
                    symbol => {
                        return Err(Error::FuncDecl(format!(
                            "Function '{name}' is previously defined as {}, now defined as {typ}",
                            symbol.typ(),
                        )));
                    }
                }
            };

            let symbol = Symbol::Func {
                typ: typ.clone(),
                defined: defined | body.is_some(),
                global,
            };

            symbols.insert(name, symbol);

            if let Some(body) = body {
                for (param, param_type) in params.iter().zip(param_types) {
                    symbols.insert(param, Symbol::LocalVar { typ: param_type });
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
            check_type(typ)?;
            let mut init_value = match init {
                Some(init) => InitialValue::from_initializer(init, typ).ok_or(Error::VarDecl(
                    format!("Static variable '{name}' has a non-constant initializer {init:?}"),
                ))?,
                None if storage_class == &StorageClass::Extern => InitialValue::NoInitializer,
                None => InitialValue::Tentative,
            };

            let mut global = storage_class != &StorageClass::Static;

            if let Some(old_symbol) = symbols.get(name) {
                match old_symbol {
                    Symbol::StaticVar {
                        typ: old_type,
                        init: old_init,
                        global: old_global,
                    } if old_type == typ => {
                        if storage_class == &StorageClass::Extern {
                            global = *old_global;
                        } else if old_global != &global {
                            return Err(Error::VarDecl(format!(
                                "Incompatible linkage for variable '{name}'"
                            )));
                        }

                        match (old_init, &mut init_value) {
                            (InitialValue::Initial(_), InitialValue::Initial(_)) => {
                                return Err(Error::VarDecl(format!(
                                    "Variable '{name}' has already been declared in this scope"
                                )));
                            }
                            (old_init @ InitialValue::Initial(_), _) => {
                                init_value = old_init.clone();
                            }
                            (InitialValue::Tentative, InitialValue::Initial(_)) => {}
                            (InitialValue::Tentative, _) => init_value = InitialValue::Tentative,
                            _ => {}
                        }
                    }
                    Symbol::LocalVar { typ: old_type } if old_type == typ => {}
                    symbol => {
                        return Err(Error::VarDecl(format!(
                            "Variable '{name}' is previously defined as {}, now defined as {}",
                            symbol.typ(),
                            typ,
                        )));
                    }
                }
            };

            let symbol = Symbol::StaticVar {
                typ: typ.clone(),
                init: init_value,
                global,
            };

            symbols.insert(name, symbol);

            if let Some(init) = init {
                check_init(init, symbols, typ)?;
            }
            Ok(())
        }
    }
}

fn check_local_var_decl(var_decl: &mut VarDecl, symbols: &mut Symbols) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, init, typ, storage_class) => {
            check_type(typ)?;
            match storage_class {
                StorageClass::Extern => {
                    if init.is_some() {
                        return Err(Error::VarDecl(format!(
                            "Extern variable '{name}' has an initializer"
                        )));
                    }
                    match symbols.get(name) {
                        Some(Symbol::Func { typ: old_typ, .. }) => {
                            return Err(Error::VarDecl(format!(
                                "Variable '{name}' is previously defined as {}, now defined as {}",
                                old_typ, typ
                            )));
                        }
                        Some(Symbol::StaticVar { typ: old_type, .. }) if old_type != typ => {
                            return Err(Error::VarDecl(format!(
                                "Variable '{name}' is previously defined as {}, now defined as {}",
                                old_type, typ
                            )));
                        }
                        Some(Symbol::LocalVar { typ: old_type, .. }) if old_type != typ => {
                            return Err(Error::VarDecl(format!(
                                "Variable '{name}' is previously defined as {}, now defined as {}",
                                old_type, typ
                            )));
                        }
                        None => {
                            symbols.insert(
                                name,
                                Symbol::StaticVar {
                                    typ: typ.clone(),
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
                        Some(init) => InitialValue::from_initializer(init, typ).ok_or(
                            Error::VarDecl(format!(
                                "Static variable '{name}' has a non-constant initializer {init:?}"
                            )),
                        )?,
                        None => InitialValue::from_none(typ),
                    };
                    symbols.insert(
                        name,
                        Symbol::StaticVar {
                            typ: typ.clone(),
                            init: init_value,
                            global: false,
                        },
                    );
                }
                StorageClass::None => {
                    symbols.insert(name, Symbol::LocalVar { typ: typ.clone() });
                    if let Some(init) = init {
                        check_init(init, symbols, typ)?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn zero_initializer(typ: &Type) -> ValidationResult<Initializer> {
    let init = match typ {
        Type::Array(element_type, size) => Initializer::Compound(
            Some(typ.clone()),
            vec![zero_initializer(element_type)?; *size],
        ),
        Type::Int => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Int(0)),
            typ: Some(Type::Int),
        }),
        Type::UInt => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::UInt(0)),
            typ: Some(Type::UInt),
        }),
        Type::Long => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Long(0)),
            typ: Some(Type::Long),
        }),
        Type::ULong => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::ULong(0)),
            typ: Some(Type::ULong),
        }),
        Type::Double => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Double(0.)),
            typ: Some(Type::Double),
        }),
        Type::Pointer(_) => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::ULong(0)),
            typ: Some(typ.clone()),
        }),
        _ => {
            return Err(Error::Type(format!(
                "Cannot create zero initializer for type {typ}"
            )));
        }
    };

    Ok(init)
}

fn check_init(
    init: &mut Initializer,
    symbols: &mut Symbols,
    target_type: &Type,
) -> ValidationResult<()> {
    match (target_type, init) {
        (_, Initializer::Single(e)) => {
            check_and_convert_exp(e, symbols)?;
            convert_by_assignment_to(e, target_type)?;
        }
        (Type::Array(element_type, size), Initializer::Compound(typ, init_list)) => {
            if init_list.len() > *size {
                return Err(Error::Type(format!("todo")));
            }
            for init in &mut *init_list {
                check_init(init, symbols, element_type)?;
            }
            while init_list.len() < *size {
                init_list.push(zero_initializer(element_type)?);
            }

            *typ = Some(target_type.clone());
        }
        _ => {
            return Err(Error::Type(format!(
                "cannot initialize a scalar object with a compound initializer"
            )));
        }
    }

    Ok(())
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
            check_and_convert_exp(exp, symbols)?;
            convert_by_assignment_to(exp, ret_type)?;
        }
        Stmt::Expression(exp) => {
            check_and_convert_exp(exp, symbols)?;
        }
        Stmt::If(cond, then, Some(else_)) => {
            check_and_convert_exp(cond, symbols)?;
            check_stmt(then, symbols, ret_type)?;
            check_stmt(else_, symbols, ret_type)?;
        }
        Stmt::If(cond, then, None) => {
            check_and_convert_exp(cond, symbols)?;
            check_stmt(then, symbols, ret_type)?;
        }
        Stmt::Labelled(_, stmt) => {
            check_stmt(stmt, symbols, ret_type)?;
        }
        Stmt::Compound(block) => {
            check_block(block, symbols, ret_type)?;
        }
        Stmt::While(cond, body, _) => {
            check_and_convert_exp(cond, symbols)?;
            check_stmt(body, symbols, ret_type)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            check_stmt(body, symbols, ret_type)?;
            check_and_convert_exp(cond, symbols)?;
        }
        Stmt::For(init, cond, post, body, _) => {
            check_for_init(init, symbols)?;
            if let Some(cond) = cond {
                check_and_convert_exp(cond, symbols)?;
            }
            if let Some(post) = post {
                check_and_convert_exp(post, symbols)?;
            }
            check_stmt(body, symbols, ret_type)?;
        }
        Stmt::Switch(cond, body, _, _) => {
            check_and_convert_exp(cond, symbols)?;
            if !cond.typ().is_integer() {
                return Err(Error::Type(format!(
                    "Switch condition must have integer type, found {}",
                    cond.typ()
                )));
            }
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
            VarDecl::VarDecl(name, _, _, _) => Err(Error::VarDecl(format!(
                "Incompatible storage class for variable '{name}'"
            ))),
        },
        ForInit::InitExp(exp) => check_and_convert_exp(exp, symbols),
        ForInit::Null => Ok(()),
    }
}

fn check_and_convert_exp(exp: &mut Exp, symbols: &mut Symbols) -> ValidationResult<()> {
    check_exp(exp, symbols)?;

    if let Type::Array(element_type, _) = exp.typ().clone() {
        let inner = std::mem::replace(exp, Exp::new(ExpKind::Const(Const::Int(0))));
        *exp = Exp {
            kind: ExpKind::AddressOf(Box::new(inner)),
            typ: Some(Type::Pointer(element_type.clone())),
        };
    }

    Ok(())
}

fn check_exp(exp: &mut Exp, symbols: &mut Symbols) -> ValidationResult<()> {
    if exp.typ.is_some() {
        return Ok(());
    }

    let Exp { kind, typ } = exp;
    match kind {
        ExpKind::Const(c) => *typ = Some(c.typ().clone()),
        ExpKind::Assignment(left, right) => {
            check_and_convert_exp(left, symbols)?;
            if !left.is_lvalue() {
                return Err(Error::Type(format!(
                    "Assignment to non-lvalue variable '{left:?}'"
                )));
            }

            check_and_convert_exp(right, symbols)?;

            let left_type = left.typ();
            convert_by_assignment_to(right, left_type)?;
            *typ = Some(left_type.clone());
        }
        ExpKind::Var(name) => {
            match symbols.get(name).unwrap() {
                Symbol::Func { .. } => {
                    return Err(Error::Type(format!(
                        "'{name}' is a function and cannot be used as a variable"
                    )));
                }
                symbol => *typ = Some(symbol.typ().clone()),
            };
        }
        ExpKind::Cast(target_type, inner) => {
            use Type::*;
            check_type(target_type)?;
            check_and_convert_exp(inner, symbols)?;
            if matches!(
                (inner.typ(), &target_type),
                (Double, Pointer(_)) | (Pointer(_), Double)
            ) {
                return Err(Error::Type(format!(
                    "Cast from {} to {target_type} is illegal",
                    inner.typ()
                )));
            }
            *typ = Some(target_type.clone());
        }
        ExpKind::Unary(op, inner) => {
            use UnaryOp::*;

            check_and_convert_exp(inner, symbols)?;

            match op {
                LogicalNot => {
                    *typ = Some(Type::Int);
                }
                BitNot => {
                    let inner_type = inner.typ().clone();
                    if inner_type == Type::Double || inner_type.is_pointer() {
                        return Err(Error::Type(format!(
                            "Complement of non-integer type {inner_type}"
                        )));
                    }
                    *typ = Some(inner_type)
                }
                Negate => {
                    let inner_type = inner.typ().clone();
                    if inner_type.is_pointer() {
                        return Err(Error::Type(format!(
                            "Negation of non-integer type {inner_type}"
                        )));
                    }
                    *typ = Some(inner_type)
                }
            };
        }
        ExpKind::Binary(op, e1, e2) => {
            use BinaryOp::*;

            check_and_convert_exp(e1, symbols)?;
            check_and_convert_exp(e2, symbols)?;

            let e1_type = e1.typ();
            let e2_type = e2.typ();

            match op {
                LogicalAnd | LogicalOr => {
                    *typ = Some(Type::Int);
                }
                LShift | RShift => {
                    if e1_type.is_integer() && e2_type.is_integer() {
                        *typ = Some(e1_type.clone())
                    } else {
                        return Err(Error::Type(format!(
                            "Both arguments for shift must have integer type: found {e1_type}, {e2_type}"
                        )));
                    }
                }
                Equal | NotEqual => {
                    let common_type = if e1_type.is_arithmetic() && e2_type.is_arithmetic() {
                        common_type_with(e1, e2)?
                    } else if e1_type.is_pointer() || e2_type.is_pointer() {
                        common_pointer_type_with(e1, e2)?
                    } else {
                        return Err(Error::Type(format!(
                            "Invalid types for equality: found {e1_type}, {e2_type}"
                        )));
                    };

                    convert_to(e1, &common_type);
                    convert_to(e2, &common_type);

                    *typ = Some(Type::Int)
                }
                GreaterThan | GreaterOrEqual | LessThan | LessOrEqual => {
                    let common_type = if e1_type.is_arithmetic() && e2_type.is_arithmetic() {
                        common_type_with(e1, e2)?
                    } else if e1_type.is_pointer() && e2_type.is_pointer() && e1_type == e2_type {
                        e1_type.clone()
                    } else {
                        return Err(Error::Type(format!(
                            "Invalid types for equality: found {e1_type}, {e2_type}"
                        )));
                    };

                    convert_to(e1, &common_type);
                    convert_to(e2, &common_type);

                    *typ = Some(Type::Int)
                }
                Add => {
                    let common_type = common_type_with(e1, e2)?;

                    if e1_type.is_arithmetic() && e2_type.is_arithmetic() {
                        convert_to(e1, &common_type);
                        convert_to(e2, &common_type);
                        *typ = Some(common_type);
                    } else if e1_type.is_pointer() && e2_type.is_integer() {
                        convert_to(e2, &Type::Long);
                        *typ = Some(e1_type.clone());
                    } else if e1_type.is_integer() && e2_type.is_pointer() {
                        convert_to(e1, &Type::Long);
                        *typ = Some(e2_type.clone());
                    } else {
                        return Err(Error::Type(format!("Invalid operands for addition")));
                    }
                }
                Subtract => {
                    let common_type = common_type_with(e1, e2)?;

                    if e1_type.is_arithmetic() && e2_type.is_arithmetic() {
                        convert_to(e1, &common_type);
                        convert_to(e2, &common_type);
                        *typ = Some(common_type);
                    } else if e1_type.is_pointer() && e2_type.is_integer() {
                        convert_to(e2, &Type::Long);
                        *typ = Some(e1_type.clone());
                    } else if e1_type.is_pointer() && e1_type == e2_type {
                        *typ = Some(Type::Long);
                    } else {
                        return Err(Error::Type(format!("Invalid operands for addition")));
                    }
                }
                Multiply | Divide | Remainder | BitAnd | BitOr | BitXor => {
                    if !e1_type.is_arithmetic() || !e2_type.is_arithmetic() {
                        return Err(Error::Type(format!(
                            "Operator {op:?} requires arithmetic operands, found {e1_type} and {e2_type}"
                        )));
                    }

                    let common_type = common_type_with(e1, e2)?;

                    if matches!(op, Remainder | BitAnd | BitOr | BitXor)
                        && common_type == Type::Double
                    {
                        return Err(Error::Type(format!(
                            "Expected integer type, found: {common_type}"
                        )));
                    }

                    convert_to(e1, &common_type);
                    convert_to(e2, &common_type);
                    *typ = Some(common_type);
                }
            }
        }
        ExpKind::Conditional(cond, then, else_) => {
            check_and_convert_exp(cond, symbols)?;
            check_and_convert_exp(then, symbols)?;
            check_and_convert_exp(else_, symbols)?;
            let common_type = if then.typ().is_pointer() || else_.typ().is_pointer() {
                common_pointer_type_with(then, else_)?
            } else {
                common_type_with(then, else_)?
            };
            convert_to(then, &common_type);
            convert_to(else_, &common_type);
            *typ = Some(common_type);
        }
        ExpKind::FuncCall(name, args) => {
            let Symbol::Func {
                typ: Type::Func(param_types, ret_type),
                ..
            } = symbols.get(name).unwrap()
            else {
                return Err(Error::Type(format!(
                    "'{name}' is a variable and cannot be called as a function"
                )));
            };
            let param_types = param_types.clone();
            let ret_type = ret_type.clone();

            if param_types.len() != args.len() {
                return Err(Error::Type(format!(
                    "Function '{name}' expects {} arguments but was called with {} arguments",
                    param_types.len(),
                    args.len()
                )));
            }

            for (arg, param_type) in args.iter_mut().zip(param_types) {
                check_and_convert_exp(arg, symbols)?;
                convert_by_assignment_to(arg, &param_type)?;
            }

            *typ = Some(*ret_type);
        }
        ExpKind::Dereference(inner) => {
            check_and_convert_exp(inner, symbols)?;
            match inner.typ() {
                Type::Pointer(referenced_type) => *typ = Some(*referenced_type.clone()),
                _ => {
                    return Err(Error::Type(format!(
                        "Trying to dereference non-pointer {typ:?}",
                    )));
                }
            }
        }
        ExpKind::AddressOf(inner) => {
            if !inner.is_lvalue() {
                return Err(Error::Type(format!(
                    "Cannot take the address of a non-lvalue {inner:?}",
                )));
            }
            check_exp(inner, symbols)?;
            *typ = Some(Type::Pointer(Box::new(inner.typ().clone())));
        }
        ExpKind::Update(op, left, right, postfix) => {
            fn make_ast_temp(typ: Type, symbols: &mut Symbols) -> Exp {
                let name = make_temporary();
                symbols.insert(name, Symbol::LocalVar { typ });
                Exp::new(ExpKind::Var(name))
            }

            fn assignment(left: Exp, right: Exp) -> Exp {
                Exp::new(ExpKind::Assignment(Box::new(left), Box::new(right)))
            }

            if !left.is_lvalue() {
                return Err(Error::Type(format!(
                    "Assignment to non-lvalue variable '{left:?}'"
                )));
            }
            check_and_convert_exp(left, symbols)?;
            let value_type = left.typ().clone();
            let ptr = make_ast_temp(Type::Pointer(Box::new(value_type.clone())), symbols);
            let deref = Exp::new(ExpKind::Dereference(Box::new(ptr.clone())));
            let left = std::mem::replace(left.as_mut(), Exp::new(ExpKind::Const(Const::Int(0))));
            let right = std::mem::replace(right.as_mut(), Exp::new(ExpKind::Const(Const::Int(0))));
            let mut expressions = vec![assignment(
                ptr,
                Exp::new(ExpKind::AddressOf(Box::new(left))),
            )];

            if *postfix {
                let old = make_ast_temp(value_type, symbols);
                expressions.push(assignment(old.clone(), deref.clone()));
                expressions.push(assignment(
                    deref,
                    Exp::new(ExpKind::Binary(*op, Box::new(old.clone()), Box::new(right))),
                ));
                expressions.push(old);
            } else {
                expressions.push(assignment(
                    deref.clone(),
                    Exp::new(ExpKind::Binary(*op, Box::new(deref), Box::new(right))),
                ));
            }

            let mut sequence = Exp::new(ExpKind::Sequence(expressions));
            check_and_convert_exp(&mut sequence, symbols)?;

            *exp = sequence;
        }
        ExpKind::Sequence(expressions) => {
            for expression in expressions.iter_mut() {
                check_and_convert_exp(expression, symbols)?;
            }
            *typ = Some(expressions.last().expect("empty sequence").typ().clone());
        }
        ExpKind::Subscript(e1, e2) => {
            check_and_convert_exp(e1, symbols)?;
            check_and_convert_exp(e2, symbols)?;

            let e1_type = e1.typ();
            let e2_type = e2.typ();

            let ptr_type;
            if e1_type.is_pointer() && e2_type.is_integer() {
                ptr_type = e1_type;
                convert_to(e2, &Type::Long)
            } else if e1_type.is_integer() && e2_type.is_pointer() {
                ptr_type = e2_type;
                convert_to(e1, &Type::Long)
            } else {
                return Err(Error::Type(format!(
                    "Subscript must have integer and pointer operands"
                )));
            }

            let Type::Pointer(referenced_type) = ptr_type else {
                return Err(Error::Type(format!("Subscript type error")));
            };
            *typ = Some(*referenced_type.clone());
        }
    };
    Ok(())
}
