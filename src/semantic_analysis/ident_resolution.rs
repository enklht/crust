use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::Ident;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

type MapEntry = (Ident, bool);

type IdentMap = FxHashMap<Ident, MapEntry>;

struct ScopeManager {
    scopes: Vec<IdentMap>,
}

impl ScopeManager {
    fn new() -> Self {
        Self {
            scopes: vec![FxHashMap::default()],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert(&mut self, name: Ident, entry: MapEntry) {
        self.scopes.last_mut().unwrap().insert(name, entry);
    }

    fn get(&self, name: &str) -> Option<&MapEntry> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn get_from_current_scope(&self, name: &str) -> Option<&MapEntry> {
        self.scopes.last().unwrap().get(name)
    }

    fn is_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(name)
    }
}

pub fn resolve_program(program: &mut Program) -> ValidationResult<()> {
    let scope_manager = &mut ScopeManager::new();
    match program {
        Program::Program(decls) => decls
            .iter_mut()
            .try_for_each(|decl| resolve_file_scope_decl(decl, scope_manager)),
    }
}

fn resolve_file_scope_decl(
    decl: &mut Decl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<()> {
    match decl {
        Decl::VarDecl(var_decl) => resolve_file_scope_var_decl(var_decl, scope_manager),
        Decl::FuncDecl(func_decl) => resolve_func_decl(func_decl, scope_manager),
    }
}

fn resolve_local_decl(decl: &mut Decl, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    match decl {
        Decl::VarDecl(var_decl) => resolve_local_var_decl(var_decl, scope_manager)?,
        Decl::FuncDecl(func_decl) => {
            if let FuncDecl::FuncDecl(name, _, Some(_), _, _) = func_decl {
                return Err(ValidationError::FuncDecl(format!(
                    "Function '{name}' cannot be defined inside another function"
                )));
            }
            resolve_func_decl(func_decl, scope_manager)?
        }
    };
    Ok(())
}

fn resolve_file_scope_var_decl(
    var_decl: &mut VarDecl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<()> {
    match var_decl {
        VarDecl::VarDecl(name, _, _, _) => {
            scope_manager.insert(name, (name, true));
        }
    };
    Ok(())
}

fn resolve_local_var_decl(
    var_decl: &mut VarDecl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<()> {
    let VarDecl::VarDecl(name, init, _, storage_class) = var_decl;

    if let Some(&(_, has_linkage)) = scope_manager.get_from_current_scope(name)
        && !(has_linkage && storage_class == &StorageClass::Extern)
    {
        return Err(ValidationError::VarDecl(format!(
            "Variable '{name}' has already been declared in this scope"
        )));
    }

    let (new_name, has_linkage) = match storage_class {
        StorageClass::Extern => (*name, true),
        _ => (make_label(name), false),
    };

    scope_manager.insert(name, (new_name, has_linkage));
    *name = new_name;

    if let Some(init) = init {
        resolve_exp(init, scope_manager)?
    }

    Ok(())
}

fn resolve_func_decl(
    func_decl: &mut FuncDecl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<()> {
    let FuncDecl::FuncDecl(name, params, body, _, _) = func_decl;

    if let Some(&(_, false)) = scope_manager.get_from_current_scope(name) {
        return Err(ValidationError::FuncDecl(format!(
            "Function '{name}' has already been declared"
        )));
    }

    scope_manager.insert(name, (name, true));

    scope_manager.enter_scope();
    params
        .iter_mut()
        .try_for_each(|param| resolve_param(param, scope_manager))?;

    if let Some(body) = body {
        resolve_block(body, scope_manager)?
    }
    scope_manager.exit_scope();

    Ok(())
}

fn resolve_param(name: &mut Ident, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    if scope_manager.is_in_current_scope(name) {
        return Err(ValidationError::VarDecl(format!(
            "Variable '{name}' has already been declared in this scope"
        )));
    }

    let unique_name = make_label(name);
    scope_manager.insert(name, (unique_name, false));
    *name = unique_name;

    Ok(())
}

fn resolve_block(block: &mut Block, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    block
        .iter_mut()
        .try_for_each(|block_item| resolve_block_item(block_item, scope_manager))
}

fn resolve_block_item(
    block_item: &mut BlockItem,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<()> {
    match block_item {
        BlockItem::S(stmt) => resolve_stmt(stmt, scope_manager),
        BlockItem::D(decl) => resolve_local_decl(decl, scope_manager),
    }
}

fn resolve_stmt(stmt: &mut Stmt, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    match stmt {
        Stmt::Return(exp) => {
            resolve_exp(exp, scope_manager)?;
        }
        Stmt::Expression(exp) => {
            resolve_exp(exp, scope_manager)?;
        }
        Stmt::If(cond, then, else_) => {
            resolve_exp(cond, scope_manager)?;
            resolve_stmt(then, scope_manager)?;
            if let Some(else_) = else_ {
                resolve_stmt(else_, scope_manager)?;
            }
        }
        Stmt::Labelled(_, stmt) => resolve_stmt(stmt, scope_manager)?,
        Stmt::Goto(_) => {}
        Stmt::Compound(block) => {
            scope_manager.enter_scope();
            resolve_block(block, scope_manager)?;
            scope_manager.exit_scope();
        }
        Stmt::While(cond, body, _) => {
            resolve_exp(cond, scope_manager)?;
            resolve_stmt(body, scope_manager)?;
        }
        Stmt::DoWhile(body, cond, _) => {
            resolve_stmt(body, scope_manager)?;
            resolve_exp(cond, scope_manager)?;
        }
        Stmt::For(init, cond, post, body, _) => {
            scope_manager.enter_scope();
            resolve_for_init(init, scope_manager)?;
            if let Some(cond) = cond {
                resolve_exp(cond, scope_manager)?;
            }
            if let Some(post) = post {
                resolve_exp(post, scope_manager)?;
            }
            resolve_stmt(body, scope_manager)?;
            scope_manager.exit_scope();
        }
        Stmt::Switch(cond, body, _, _) => {
            resolve_exp(cond, scope_manager)?;
            resolve_stmt(body, scope_manager)?;
        }
        _ => {}
    };
    Ok(())
}

fn resolve_for_init(init: &mut ForInit, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    match init {
        ForInit::InitExp(exp) => resolve_exp(exp, scope_manager),
        ForInit::InitDecl(var_decl) => resolve_local_var_decl(var_decl, scope_manager),
        ForInit::Null => Ok(()),
    }
}

fn resolve_exp(exp: &mut Exp, scope_manager: &mut ScopeManager) -> ValidationResult<()> {
    match &mut exp.kind {
        ExpKind::Const(_) => {}
        ExpKind::Assignment(left, right) => {
            resolve_exp(left, scope_manager)?;
            resolve_exp(right, scope_manager)?;
        }
        ExpKind::Var(name) => match scope_manager.get(name) {
            Some((unique_name, _)) => *name = unique_name,
            None => {
                return Err(ValidationError::VarDecl(format!(
                    "Cannot find variable '{name}' in this scope"
                )));
            }
        },
        ExpKind::Cast(_, exp) => resolve_exp(exp, scope_manager)?,
        ExpKind::Unary(_, exp) => resolve_exp(exp, scope_manager)?,
        ExpKind::Binary(_, e1, e2) => {
            resolve_exp(e1, scope_manager)?;
            resolve_exp(e2, scope_manager)?;
        }
        ExpKind::Conditional(cond, then, else_) => {
            resolve_exp(cond, scope_manager)?;
            resolve_exp(then, scope_manager)?;
            resolve_exp(else_, scope_manager)?;
        }
        ExpKind::FuncCall(name, args) => {
            let Some((new_name, _)) = scope_manager.get(name) else {
                return Err(ValidationError::FuncDecl(format!(
                    "Cannot find function '{name}' in this scope"
                )));
            };

            *name = new_name;
            args.iter_mut()
                .try_for_each(|arg| resolve_exp(arg, scope_manager))?;
        }
        ExpKind::Dereference(inner) => {
            resolve_exp(inner, scope_manager)?;
        }
        ExpKind::AddressOf(inner) => {
            resolve_exp(inner, scope_manager)?;
        }
        ExpKind::Sequence(bindings, effects, result) => {
            scope_manager.enter_scope();
            for (name, init) in bindings {
                resolve_exp(init, scope_manager)?;
                scope_manager.insert(name, (name, false));
            }
            for effect in effects {
                resolve_exp(effect, scope_manager)?;
            }
            resolve_exp(result, scope_manager)?;
            scope_manager.exit_scope();
        }
    }
    Ok(())
}
