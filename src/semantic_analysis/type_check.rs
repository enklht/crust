use super::ValidationError as Error;
use super::ValidationResult;
use crate::models::ast::*;
use crate::models::symbol::*;
use crate::models::typ::{TypeId, TypeKind, Types};
use crate::utils::make_temporary;
use std::cmp::Ordering;

pub fn common_type_with(left: &Exp, right: &Exp, types: &Types) -> ValidationResult<TypeId> {
    let left_type = left.typ();
    let right_type = right.typ();

    if left_type == right_type {
        Ok(left_type)
    } else if left_type == types.double || right_type == types.double {
        Ok(types.double)
    } else {
        match types
            .size(left_type)
            .ok_or(Error::Type(format!(
                "{left_type} has no size",
                left_type = types.display(left_type)
            )))?
            .cmp(&types.size(right_type).ok_or(Error::Type(format!(
                "{right_type} has no size",
                right_type = types.display(right_type)
            )))?) {
            Ordering::Equal if types.is_signed(left_type) => Ok(right_type),
            Ordering::Equal => Ok(left_type),
            Ordering::Greater => Ok(left_type),
            Ordering::Less => Ok(right_type),
        }
    }
}

pub fn common_pointer_type_with(
    left: &Exp,
    right: &Exp,
    types: &Types,
) -> ValidationResult<TypeId> {
    let left_type = left.typ();
    let right_type = right.typ();

    if !types.is_pointer(left_type) && !types.is_pointer(right_type) {
        Err(Error::Type(format!(
            "Expected a pointer operand, found '{left_type}' and '{right_type}'",
            left_type = types.display(left_type),
            right_type = types.display(right_type)
        )))
    } else if left_type == right_type {
        Ok(left_type)
    } else if left.is_null_pointer_constant() {
        Ok(right_type)
    } else if right.is_null_pointer_constant() {
        Ok(left_type)
    } else {
        Err(Error::Type(format!(
            "Incompatible pointer types: '{left_type}' and '{right_type}'",
            left_type = types.display(left_type),
            right_type = types.display(right_type)
        )))
    }
}

pub fn convert_to(exp: &mut Exp, typ: TypeId) {
    if exp.typ() != typ {
        let inner = std::mem::replace(exp, Exp::new(ExpKind::Const(Const::Int(0))));
        *exp = Exp {
            kind: ExpKind::Cast(typ, Box::new(inner)),
            typ: Some(typ),
        };
    }
}

pub fn convert_by_assignment_to(
    exp: &mut Exp,
    target_type: TypeId,
    types: &Types,
) -> ValidationResult<()> {
    match exp.typ() {
        typ if typ == target_type => {}
        typ if types.is_arithmetic(typ) && types.is_arithmetic(target_type) => {
            convert_to(exp, target_type)
        }
        _ if exp.is_null_pointer_constant() && types.is_pointer(target_type) => {
            convert_to(exp, target_type)
        }
        _ => {
            return Err(Error::Type(format!(
                "Cannot convert '{source_type}' to '{target_type}' by assignment",
                source_type = types.display(exp.typ()),
                target_type = types.display(target_type)
            )));
        }
    }
    Ok(())
}

pub fn check_program(program: &mut Program, types: &mut Types) -> ValidationResult<Symbols> {
    let mut symbols = Symbols::default();
    match program {
        Program::Program(decls) => {
            decls
                .iter_mut()
                .try_for_each(|decl| check_file_scope_decl(decl, &mut symbols, types))?;
        }
    }
    Ok(symbols)
}

fn check_type(typ: TypeId, types: &Types) -> ValidationResult<()> {
    match types.kind(typ) {
        TypeKind::Array(element_type, _) => {
            check_type(*element_type, types)?;
            if !types
                .size(typ)
                .is_some_and(|size| size <= isize::MAX as usize)
            {
                return Err(Error::Type(
                    "Array size exceeds the maximum supported object size".to_string(),
                ));
            }
        }
        TypeKind::Pointer(referenced_type) => check_type(*referenced_type, types)?,
        TypeKind::Func(param_types, ret_type) => {
            for param_type in param_types {
                check_type(*param_type, types)?;
            }
            check_type(*ret_type, types)?;
        }
        _ => {}
    }
    Ok(())
}

fn check_file_scope_decl(
    decl: &mut Decl,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, false, types),
        Decl::VarDecl(var_decl) => check_file_scope_var_decl(var_decl, symbols, types),
    }
}

fn check_local_decl(
    decl: &mut Decl,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    match decl {
        Decl::FuncDecl(func_decl) => check_func_decl(func_decl, symbols, true, types),
        Decl::VarDecl(var_decl) => check_local_var_decl(var_decl, symbols, types),
    }
}

fn check_func_decl(
    func_decl: &mut FuncDecl,
    symbols: &mut Symbols,
    block_scope: bool,
    types: &mut Types,
) -> ValidationResult<()> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, body, typ, storage_class) => {
            check_type(*typ, types)?;
            let TypeKind::Func(param_types, ret_type) = types.kind(*typ).clone() else {
                unreachable!()
            };

            if types.is_array(ret_type) {
                return Err(Error::Type("A function cannot return an array".to_string()));
            }

            let param_types: Vec<TypeId> = param_types
                .iter()
                .map(|t| match *types.kind(*t) {
                    TypeKind::Array(element_type, _) => {
                        types.intern(TypeKind::Pointer(element_type))
                    }
                    _ => *t,
                })
                .collect();

            *typ = types.intern(TypeKind::Func(param_types.clone(), ret_type));

            if block_scope && storage_class == &StorageClass::Static {
                return Err(Error::FuncDecl(format!(
                    "Function '{name}' cannot be declared 'static' inside a block"
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
                                "Function '{name}' is defined more than once",
                            )));
                        }
                        if *old_global && storage_class == &StorageClass::Static {
                            return Err(Error::FuncDecl(format!(
                                "Function '{name}' was declared with external linkage and cannot be redeclared 'static'"
                            )));
                        }

                        defined = *old_defined;
                        global = *old_global;
                    }
                    symbol => {
                        return Err(Error::FuncDecl(format!(
                            "Conflicting types for function '{name}': previously '{}', now '{}'",
                            types.display(symbol.typ()),
                            types.display(*typ),
                        )));
                    }
                }
            };

            let symbol = Symbol::Func {
                typ: *typ,
                defined: defined | body.is_some(),
                global,
            };

            symbols.insert(name, symbol);

            if let Some(body) = body {
                for (param, param_type) in params.iter().zip(param_types) {
                    symbols.insert(param, Symbol::LocalVar { typ: param_type });
                }
                check_block(body, symbols, ret_type, types)?;
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
            check_type(*typ, types)?;
            let mut init_value = match init {
                Some(init) => InitialValue::from_initializer(init, *typ, types).ok_or_else(|| {
                    Error::VarDecl(format!(
                        "Initializer for static variable '{name}' must be a constant compatible with '{}'",
                        types.display(*typ)
                    ))
                })?,
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
                                "Variable '{name}' has conflicting 'static' and external declarations"
                            )));
                        }

                        match (old_init, &mut init_value) {
                            (InitialValue::Initial(_), InitialValue::Initial(_)) => {
                                return Err(Error::VarDecl(format!(
                                    "Variable '{name}' has more than one definition with an initializer"
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
                            "Conflicting types for variable '{name}': previously '{}', now '{}'",
                            types.display(symbol.typ()),
                            types.display(*typ),
                        )));
                    }
                }
            };

            let symbol = Symbol::StaticVar {
                typ: *typ,
                init: init_value,
                global,
            };

            symbols.insert(name, symbol);

            if let Some(init) = init {
                check_init(init, symbols, *typ, types)?;
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
            let name_in_source = name.split('.').next().unwrap();
            check_type(*typ, types)?;
            match storage_class {
                StorageClass::Extern => {
                    if init.is_some() {
                        return Err(Error::VarDecl(format!(
                            "Block-scope 'extern' variable '{name_in_source}' cannot have an initializer"
                        )));
                    }
                    match symbols.get(name) {
                        Some(Symbol::Func { typ: old_typ, .. }) => {
                            return Err(Error::VarDecl(format!(
                                "Conflicting types for variable '{name_in_source}': previously '{}', now '{}'",
                                types.display(*old_typ),
                                types.display(*typ)
                            )));
                        }
                        Some(Symbol::StaticVar { typ: old_type, .. }) if old_type != typ => {
                            return Err(Error::VarDecl(format!(
                                "Conflicting types for variable '{name_in_source}': previously '{}', now '{}'",
                                types.display(*old_type),
                                types.display(*typ)
                            )));
                        }
                        Some(Symbol::LocalVar { typ: old_type, .. }) if old_type != typ => {
                            return Err(Error::VarDecl(format!(
                                "Conflicting types for variable '{name_in_source}': previously '{}', now '{}'",
                                types.display(*old_type),
                                types.display(*typ)
                            )));
                        }
                        None => {
                            symbols.insert(
                                name,
                                Symbol::StaticVar {
                                    typ: *typ,
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
                        Some(init) => InitialValue::from_initializer(init, *typ, types).ok_or_else(|| {
                            Error::VarDecl(format!(
                                "Initializer for static variable '{name_in_source}' must be a constant compatible with '{}'",
                                types.display(*typ)
                            ))
                        })?,
                        None => InitialValue::from_none(*typ, types),
                    };
                    symbols.insert(
                        name,
                        Symbol::StaticVar {
                            typ: *typ,
                            init: init_value,
                            global: false,
                        },
                    );
                }
                StorageClass::None => {
                    symbols.insert(name, Symbol::LocalVar { typ: *typ });
                    if let Some(init) = init {
                        check_init(init, symbols, *typ, types)?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn zero_initializer(typ: TypeId, types: &Types) -> ValidationResult<Initializer> {
    let init = match types.kind(typ) {
        TypeKind::Array(element_type, size) => Initializer::Compound(
            Some(typ),
            vec![zero_initializer(*element_type, types)?; *size],
        ),
        TypeKind::Int => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Int(0)),
            typ: Some(types.int),
        }),
        TypeKind::UInt => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::UInt(0)),
            typ: Some(types.uint),
        }),
        TypeKind::Long => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Long(0)),
            typ: Some(types.long),
        }),
        TypeKind::ULong => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::ULong(0)),
            typ: Some(types.ulong),
        }),
        TypeKind::Double => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::Double(0.)),
            typ: Some(types.double),
        }),
        TypeKind::Pointer(_) => Initializer::Single(Exp {
            kind: ExpKind::Const(Const::ULong(0)),
            typ: Some(typ),
        }),
        _ => {
            return Err(Error::Type(format!(
                "Cannot create zero initializer for type {typ}",
                typ = types.display(typ)
            )));
        }
    };

    Ok(init)
}

fn check_init(
    init: &mut Initializer,
    symbols: &mut Symbols,
    target_type: TypeId,
    types: &mut Types,
) -> ValidationResult<()> {
    match (types.kind(target_type), init) {
        (_, Initializer::Single(e)) => {
            check_and_convert_exp(e, symbols, types)?;
            convert_by_assignment_to(e, target_type, types)?;
        }
        (TypeKind::Array(element_type, size), Initializer::Compound(typ, init_list)) => {
            let (element_type, size) = (*element_type, *size);
            if init_list.len() > size {
                return Err(Error::Type(format!(
                    "Too many array initializers: expected at most {size}, found {}",
                    init_list.len()
                )));
            }
            for init in &mut *init_list {
                check_init(init, symbols, element_type, types)?;
            }
            while init_list.len() < size {
                init_list.push(zero_initializer(element_type, types)?);
            }

            *typ = Some(target_type);
        }
        _ => {
            return Err(Error::Type(format!(
                "Cannot initialize scalar type '{}' with a brace-enclosed initializer",
                types.display(target_type)
            )));
        }
    }

    Ok(())
}

fn check_block(
    block: &mut Block,
    symbols: &mut Symbols,
    ret_type: TypeId,
    types: &mut Types,
) -> ValidationResult<()> {
    block
        .iter_mut()
        .try_for_each(|block_item| check_block_item(block_item, symbols, ret_type, types))
}

fn check_block_item(
    block_item: &mut BlockItem,
    symbols: &mut Symbols,
    ret_type: TypeId,
    types: &mut Types,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => check_stmt(stmt, symbols, ret_type, types),
        BlockItem::D(decl) => check_local_decl(decl, symbols, types),
    }
}

fn check_stmt(
    stmt: &mut Stmt,
    symbols: &mut Symbols,
    ret_type: TypeId,
    types: &mut Types,
) -> ValidationResult<()> {
    match stmt {
        Stmt::Return(exp) => {
            check_and_convert_exp(exp, symbols, types)?;
            convert_by_assignment_to(exp, ret_type, types)?;
        }
        Stmt::Expression(exp) => {
            check_and_convert_exp(exp, symbols, types)?;
        }
        Stmt::If(cond, then, Some(else_)) => {
            check_and_convert_exp(cond, symbols, types)?;
            check_stmt(then, symbols, ret_type, types)?;
            check_stmt(else_, symbols, ret_type, types)?;
        }
        Stmt::If(cond, then, None) => {
            check_and_convert_exp(cond, symbols, types)?;
            check_stmt(then, symbols, ret_type, types)?;
        }
        Stmt::Labelled(_, stmt) => {
            check_stmt(stmt, symbols, ret_type, types)?;
        }
        Stmt::Compound(block) => {
            check_block(block, symbols, ret_type, types)?;
        }
        Stmt::While(cond, body, _) => {
            check_and_convert_exp(cond, symbols, types)?;
            check_stmt(body, symbols, ret_type, types)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            check_stmt(body, symbols, ret_type, types)?;
            check_and_convert_exp(cond, symbols, types)?;
        }
        Stmt::For(init, cond, post, body, _) => {
            check_for_init(init, symbols, types)?;
            if let Some(cond) = cond {
                check_and_convert_exp(cond, symbols, types)?;
            }
            if let Some(post) = post {
                check_and_convert_exp(post, symbols, types)?;
            }
            check_stmt(body, symbols, ret_type, types)?;
        }
        Stmt::Switch(cond, body, _, _) => {
            check_and_convert_exp(cond, symbols, types)?;
            if !types.is_integer(cond.typ()) {
                return Err(Error::Type(format!(
                    "Switch condition must have integer type, found {}",
                    types.display(cond.typ())
                )));
            }
            check_stmt(body, symbols, ret_type, types)?;
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
            VarDecl::VarDecl(name, _, _, _) => Err(Error::VarDecl(format!(
                "Variable '{}' in a 'for' initializer cannot be declared 'static' or 'extern'",
                name.split('.').next().unwrap()
            ))),
        },
        ForInit::InitExp(exp) => check_and_convert_exp(exp, symbols, types),
        ForInit::Null => Ok(()),
    }
}

fn check_and_convert_exp(
    exp: &mut Exp,
    symbols: &mut Symbols,
    types: &mut Types,
) -> ValidationResult<()> {
    check_exp(exp, symbols, types)?;

    if let TypeKind::Array(element_type, _) = *types.kind(exp.typ()) {
        let inner = std::mem::replace(exp, Exp::new(ExpKind::Const(Const::Int(0))));
        *exp = Exp {
            kind: ExpKind::AddressOf(Box::new(inner)),
            typ: Some(types.intern(TypeKind::Pointer(element_type))),
        };
    }

    Ok(())
}

fn check_exp(exp: &mut Exp, symbols: &mut Symbols, types: &mut Types) -> ValidationResult<()> {
    if exp.typ.is_some() {
        return Ok(());
    }

    let Exp { kind, typ } = exp;
    match kind {
        ExpKind::Const(c) => *typ = Some(c.typ(types)),
        ExpKind::Assignment(left, right) => {
            check_and_convert_exp(left, symbols, types)?;
            if !left.is_lvalue() {
                return Err(Error::Type(
                    "The left operand of an assignment must be a modifiable object".to_string(),
                ));
            }

            check_and_convert_exp(right, symbols, types)?;

            let left_type = left.typ();
            convert_by_assignment_to(right, left_type, types)?;
            *typ = Some(left_type);
        }
        ExpKind::Var(name) => {
            match symbols.get(name).unwrap() {
                Symbol::Func { .. } => {
                    return Err(Error::Type(format!(
                        "'{name}' is a function and cannot be used as a variable"
                    )));
                }
                symbol => *typ = Some(symbol.typ()),
            };
        }
        ExpKind::Cast(target_type, inner) => {
            use TypeKind::*;
            check_type(*target_type, types)?;
            check_and_convert_exp(inner, symbols, types)?;
            if matches!(
                (types.kind(inner.typ()), types.kind(*target_type)),
                (Double, Pointer(_)) | (Pointer(_), Double)
            ) {
                return Err(Error::Type(format!(
                    "Cannot cast '{}' to '{}'",
                    types.display(inner.typ()),
                    types.display(*target_type)
                )));
            }
            *typ = Some(*target_type);
        }
        ExpKind::Unary(op, inner) => {
            use UnaryOp::*;

            check_and_convert_exp(inner, symbols, types)?;

            match op {
                LogicalNot => {
                    *typ = Some(types.int);
                }
                BitNot => {
                    let inner_type = inner.typ();
                    if inner_type == types.double || types.is_pointer(inner_type) {
                        return Err(Error::Type(format!(
                            "Operator '~' requires an integer operand, found '{inner_type}'",
                            inner_type = types.display(inner_type)
                        )));
                    }
                    *typ = Some(inner_type)
                }
                Negate => {
                    let inner_type = inner.typ();
                    if types.is_pointer(inner_type) {
                        return Err(Error::Type(format!(
                            "Operator '-' requires an arithmetic operand, found '{inner_type}'",
                            inner_type = types.display(inner_type)
                        )));
                    }
                    *typ = Some(inner_type)
                }
            };
        }
        ExpKind::Binary(op, e1, e2) => {
            use BinaryOp::*;

            check_and_convert_exp(e1, symbols, types)?;
            check_and_convert_exp(e2, symbols, types)?;

            let e1_type = e1.typ();
            let e2_type = e2.typ();

            match op {
                LogicalAnd | LogicalOr => {
                    *typ = Some(types.int);
                }
                LShift | RShift => {
                    if types.is_integer(e1_type) && types.is_integer(e2_type) {
                        *typ = Some(e1_type)
                    } else {
                        return Err(Error::Type(format!(
                            "Operator '{op}' requires integer operands, found '{e1_type}' and '{e2_type}'",
                            e1_type = types.display(e1_type),
                            e2_type = types.display(e2_type)
                        )));
                    }
                }
                Equal | NotEqual => {
                    let common_type =
                        if types.is_arithmetic(e1_type) && types.is_arithmetic(e2_type) {
                            common_type_with(e1, e2, types)?
                        } else if types.is_pointer(e1_type) || types.is_pointer(e2_type) {
                            common_pointer_type_with(e1, e2, types)?
                        } else {
                            return Err(Error::Type(format!(
                                "Operator '{op}' cannot compare '{e1_type}' and '{e2_type}'",
                                e1_type = types.display(e1_type),
                                e2_type = types.display(e2_type)
                            )));
                        };

                    convert_to(e1, common_type);
                    convert_to(e2, common_type);

                    *typ = Some(types.int)
                }
                GreaterThan | GreaterOrEqual | LessThan | LessOrEqual => {
                    let common_type =
                        if types.is_arithmetic(e1_type) && types.is_arithmetic(e2_type) {
                            common_type_with(e1, e2, types)?
                        } else if types.is_pointer(e1_type)
                            && types.is_pointer(e2_type)
                            && e1_type == e2_type
                        {
                            e1_type
                        } else {
                            return Err(Error::Type(format!(
                                "Operator '{op}' cannot compare '{e1_type}' and '{e2_type}'",
                                e1_type = types.display(e1_type),
                                e2_type = types.display(e2_type)
                            )));
                        };

                    convert_to(e1, common_type);
                    convert_to(e2, common_type);

                    *typ = Some(types.int)
                }
                Add => {
                    let common_type = common_type_with(e1, e2, types)?;

                    if types.is_arithmetic(e1_type) && types.is_arithmetic(e2_type) {
                        convert_to(e1, common_type);
                        convert_to(e2, common_type);
                        *typ = Some(common_type);
                    } else if types.is_pointer(e1_type) && types.is_integer(e2_type) {
                        convert_to(e2, types.long);
                        *typ = Some(e1_type);
                    } else if types.is_integer(e1_type) && types.is_pointer(e2_type) {
                        convert_to(e1, types.long);
                        *typ = Some(e2_type);
                    } else {
                        return Err(Error::Type(format!(
                            "Operator '{op}' cannot combine '{}' and '{}'",
                            types.display(e1_type),
                            types.display(e2_type)
                        )));
                    }
                }
                Subtract => {
                    let common_type = common_type_with(e1, e2, types)?;

                    if types.is_arithmetic(e1_type) && types.is_arithmetic(e2_type) {
                        convert_to(e1, common_type);
                        convert_to(e2, common_type);
                        *typ = Some(common_type);
                    } else if types.is_pointer(e1_type) && types.is_integer(e2_type) {
                        convert_to(e2, types.long);
                        *typ = Some(e1_type);
                    } else if types.is_pointer(e1_type) && e1_type == e2_type {
                        *typ = Some(types.long);
                    } else {
                        return Err(Error::Type(format!(
                            "Operator '{op}' cannot combine '{}' and '{}'",
                            types.display(e1_type),
                            types.display(e2_type)
                        )));
                    }
                }
                Multiply | Divide | Remainder | BitAnd | BitOr | BitXor => {
                    if !types.is_arithmetic(e1_type) || !types.is_arithmetic(e2_type) {
                        return Err(Error::Type(format!(
                            "Operator '{op}' requires arithmetic operands, found '{e1_type}' and '{e2_type}'",
                            e1_type = types.display(e1_type),
                            e2_type = types.display(e2_type)
                        )));
                    }

                    let common_type = common_type_with(e1, e2, types)?;

                    if matches!(op, Remainder | BitAnd | BitOr | BitXor)
                        && common_type == types.double
                    {
                        return Err(Error::Type(format!(
                            "Operator '{op}' requires integer operands, found '{common_type}'",
                            common_type = types.display(common_type)
                        )));
                    }

                    convert_to(e1, common_type);
                    convert_to(e2, common_type);
                    *typ = Some(common_type);
                }
            }
        }
        ExpKind::Conditional(cond, then, else_) => {
            check_and_convert_exp(cond, symbols, types)?;
            check_and_convert_exp(then, symbols, types)?;
            check_and_convert_exp(else_, symbols, types)?;
            let common_type = if types.is_pointer(then.typ()) || types.is_pointer(else_.typ()) {
                common_pointer_type_with(then, else_, types)?
            } else {
                common_type_with(then, else_, types)?
            };
            convert_to(then, common_type);
            convert_to(else_, common_type);
            *typ = Some(common_type);
        }
        ExpKind::FuncCall(name, args) => {
            let Symbol::Func { typ: func_type, .. } = symbols.get(name).unwrap() else {
                return Err(Error::Type(format!(
                    "'{}' is a variable and cannot be called as a function",
                    name.split('.').next().unwrap()
                )));
            };
            let TypeKind::Func(param_types, ret_type) = types.kind(*func_type).clone() else {
                unreachable!()
            };

            if param_types.len() != args.len() {
                return Err(Error::Type(format!(
                    "Wrong number of arguments to function '{name}': expected {}, found {}",
                    param_types.len(),
                    args.len()
                )));
            }

            for (i, (arg, param_type)) in args.iter_mut().zip(param_types).enumerate() {
                check_and_convert_exp(arg, symbols, types)?;
                convert_by_assignment_to(arg, param_type, types).map_err(|err| {
                    Error::Type(format!("Argument {} of function '{name}': {err}", i + 1))
                })?;
            }

            *typ = Some(ret_type);
        }
        ExpKind::Dereference(inner) => {
            check_and_convert_exp(inner, symbols, types)?;
            match types.kind(inner.typ()) {
                TypeKind::Pointer(referenced_type) => *typ = Some(*referenced_type),
                _ => {
                    return Err(Error::Type(format!(
                        "Operator '*' requires a pointer operand, found '{}'",
                        types.display(inner.typ())
                    )));
                }
            }
        }
        ExpKind::AddressOf(inner) => {
            if !inner.is_lvalue() {
                return Err(Error::Type(
                    "Operator '&' requires an object with an address".to_string(),
                ));
            }
            check_exp(inner, symbols, types)?;
            *typ = Some(types.intern(TypeKind::Pointer(inner.typ())));
        }
        ExpKind::Update(op, left, right, postfix) => {
            fn make_ast_temp(typ: TypeId, symbols: &mut Symbols) -> Exp {
                let name = make_temporary();
                symbols.insert(name, Symbol::LocalVar { typ });
                Exp::new(ExpKind::Var(name))
            }

            fn assignment(left: Exp, right: Exp) -> Exp {
                Exp::new(ExpKind::Assignment(Box::new(left), Box::new(right)))
            }

            if !left.is_lvalue() {
                return Err(Error::Type(
                    "The left operand of an assignment must be a modifiable object".to_string(),
                ));
            }
            check_and_convert_exp(left, symbols, types)?;
            let value_type = left.typ();
            let ptr = make_ast_temp(types.intern(TypeKind::Pointer(value_type)), symbols);
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
            check_and_convert_exp(&mut sequence, symbols, types)?;

            *exp = sequence;
        }
        ExpKind::Sequence(expressions) => {
            for expression in expressions.iter_mut() {
                check_and_convert_exp(expression, symbols, types)?;
            }
            *typ = Some(expressions.last().expect("empty sequence").typ());
        }
        ExpKind::Subscript(e1, e2) => {
            check_and_convert_exp(e1, symbols, types)?;
            check_and_convert_exp(e2, symbols, types)?;

            let e1_type = e1.typ();
            let e2_type = e2.typ();

            let ptr_type;
            if types.is_pointer(e1_type) && types.is_integer(e2_type) {
                ptr_type = e1_type;
                convert_to(e2, types.long)
            } else if types.is_integer(e1_type) && types.is_pointer(e2_type) {
                ptr_type = e2_type;
                convert_to(e1, types.long)
            } else {
                return Err(Error::Type(format!(
                    "Subscript requires a pointer and an integer, found '{}' and '{}'",
                    types.display(e1_type),
                    types.display(e2_type)
                )));
            }

            let TypeKind::Pointer(referenced_type) = types.kind(ptr_type) else {
                return Err(Error::Type("Subscript type error".to_string()));
            };
            *typ = Some(*referenced_type);
        }
    };
    Ok(())
}
