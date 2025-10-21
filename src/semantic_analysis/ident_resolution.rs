use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

type MapEntry = (String, bool);

type IdentMap = FxHashMap<String, MapEntry>;

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

    fn insert(&mut self, name: String, entry: MapEntry) {
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

pub fn resolve_program(program: Program<Plain>) -> ValidationResult<Program<Plain>> {
    let scope_manager = &mut ScopeManager::new();
    let program = match program {
        Program::Program(decl) => Program::Program(
            decl.into_iter()
                .map(|decl| resolve_file_scope_decl(decl, scope_manager))
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn resolve_file_scope_decl(
    decl: Decl<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<Decl<Plain>> {
    let decl = match decl {
        Decl::VarDecl(var_decl) => {
            Decl::VarDecl(resolve_file_scope_var_decl(var_decl, scope_manager)?)
        }
        Decl::FuncDecl(func_decl) => Decl::FuncDecl(resolve_func_decl(func_decl, scope_manager)?),
    };
    Ok(decl)
}

fn resolve_local_decl(
    decl: Decl<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<Decl<Plain>> {
    let decl = match decl {
        Decl::VarDecl(var_decl) => Decl::VarDecl(resolve_local_var_decl(var_decl, scope_manager)?),
        Decl::FuncDecl(func_decl) => {
            if let FuncDecl::FuncDecl(name, _, Some(_), _) = func_decl {
                return Err(ValidationError::NestedFuncDef(name));
            }
            Decl::FuncDecl(resolve_func_decl(func_decl, scope_manager)?)
        }
    };
    Ok(decl)
}

fn resolve_file_scope_var_decl(
    var_decl: VarDecl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<VarDecl> {
    let var_decl = match var_decl.clone() {
        VarDecl::VarDecl(name, _, _) => {
            scope_manager.insert(name.clone(), (name, true));
            var_decl
        }
    };
    Ok(var_decl)
}

fn resolve_local_var_decl(
    var_decl: VarDecl,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<VarDecl> {
    let var_decl = match var_decl {
        VarDecl::VarDecl(name, init, storage_class) => {
            if let Some(&(_, has_linkage)) = scope_manager.get_from_current_scope(&name)
                && !(has_linkage && storage_class == StorageClass::Extern)
            {
                return Err(ValidationError::DuplicateVarDecl(name));
            }

            let (new_name, has_linkage) = match storage_class {
                StorageClass::Extern => (name.clone(), true),
                _ => (make_label(&name), false),
            };

            scope_manager.insert(name, (new_name.clone(), has_linkage));

            VarDecl::VarDecl(
                new_name,
                init.map(|exp| resolve_exp(exp, scope_manager))
                    .transpose()?,
                storage_class,
            )
        }
    };
    Ok(var_decl)
}

fn resolve_func_decl(
    func_decl: FuncDecl<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<FuncDecl<Plain>> {
    let func_decl = match func_decl {
        FuncDecl::FuncDecl(name, params, body, storage_class) => {
            if let Some(&(_, false)) = scope_manager.get_from_current_scope(&name) {
                return Err(ValidationError::DuplicateFuncDecl(name));
            }

            scope_manager.insert(name.clone(), (name.clone(), true));

            scope_manager.enter_scope();
            let new_params = params
                .into_iter()
                .map(|param| resolve_param(param, scope_manager))
                .collect::<Result<_, _>>()?;

            let new_body = body
                .map(|block| resolve_block(block, scope_manager))
                .transpose()?;
            scope_manager.exit_scope();

            FuncDecl::FuncDecl(name, new_params, new_body, storage_class)
        }
    };
    Ok(func_decl)
}

fn resolve_param(name: String, scope_manager: &mut ScopeManager) -> ValidationResult<String> {
    if scope_manager.is_in_current_scope(&name) {
        return Err(ValidationError::DuplicateVarDecl(name));
    }

    let unique_name = make_label(&name);
    scope_manager.insert(name, (unique_name.clone(), false));

    Ok(unique_name)
}

fn resolve_block(
    block: Block<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<Block<Plain>> {
    block
        .into_iter()
        .map(|block_item| resolve_block_item(block_item, scope_manager))
        .collect::<ValidationResult<_>>()
}

fn resolve_block_item(
    block_item: BlockItem<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<BlockItem<Plain>> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, scope_manager)?),
        BlockItem::D(decl) => BlockItem::D(resolve_local_decl(decl, scope_manager)?),
    };
    Ok(block_item)
}

fn resolve_stmt(
    stmt: Stmt<Plain>,
    scope_manager: &mut ScopeManager,
) -> ValidationResult<Stmt<Plain>> {
    let stmt = match stmt {
        Stmt::Return(exp) => Stmt::Return(resolve_exp(exp, scope_manager)?),
        Stmt::Expression(exp) => Stmt::Expression(resolve_exp(exp, scope_manager)?),
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            resolve_exp(cond, scope_manager)?,
            Box::new(resolve_stmt(*then, scope_manager)?),
            Some(Box::new(resolve_stmt(*else_, scope_manager)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(
            resolve_exp(cond, scope_manager)?,
            Box::new(resolve_stmt(*then, scope_manager)?),
            None,
        ),
        Stmt::Labelled(label, stmt) => {
            Stmt::Labelled(label, Box::new(resolve_stmt(*stmt, scope_manager)?))
        }
        Stmt::Goto(label) => Stmt::Goto(label),
        Stmt::Compound(block) => {
            scope_manager.enter_scope();
            let resolved_block = resolve_block(block, scope_manager)?;
            scope_manager.exit_scope();
            Stmt::Compound(resolved_block)
        }
        Stmt::While(cond, body, label) => Stmt::While(
            resolve_exp(cond, scope_manager)?,
            Box::new(resolve_stmt(*body, scope_manager)?),
            label,
        ),
        Stmt::DoWhile(body, cond, label) => Stmt::DoWhile(
            Box::new(resolve_stmt(*body, scope_manager)?),
            resolve_exp(cond, scope_manager)?,
            label,
        ),
        Stmt::For(init, cond, post, body, label) => {
            scope_manager.enter_scope();
            let init = resolve_for_init(init, scope_manager)?;
            let cond = cond
                .map(|cond| resolve_exp(cond, scope_manager))
                .transpose()?;
            let post = post
                .map(|post| resolve_exp(post, scope_manager))
                .transpose()?;
            let body = Box::new(resolve_stmt(*body, scope_manager)?);
            scope_manager.exit_scope();
            Stmt::For(init, cond, post, body, label)
        }
        Stmt::Switch(cond, body, label, ()) => {
            let cond = resolve_exp(cond, scope_manager)?;
            let body = Box::new(resolve_stmt(*body, scope_manager)?);
            Stmt::Switch(cond, body, label, ())
        }
        // Break, Continue, Null
        other => other,
    };
    Ok(stmt)
}

fn resolve_for_init(init: ForInit, scope_manager: &mut ScopeManager) -> ValidationResult<ForInit> {
    let init = match init {
        ForInit::InitExp(exp) => ForInit::InitExp(resolve_exp(exp, scope_manager)?),
        ForInit::InitDecl(var_decl) => {
            ForInit::InitDecl(resolve_local_var_decl(var_decl, scope_manager)?)
        }
        ForInit::Null => ForInit::Null,
    };
    Ok(init)
}

fn resolve_exp(exp: Exp, scope_manager: &mut ScopeManager) -> ValidationResult<Exp> {
    let exp = match exp {
        Exp::Const(i) => Exp::Const(i),
        Exp::Assignment(left, right) => match *left {
            left @ Exp::Var(_) => Exp::Assignment(
                Box::new(resolve_exp(left, scope_manager)?),
                Box::new(resolve_exp(*right, scope_manager)?),
            ),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Var(name) => match scope_manager.get(&name) {
            Some((name, _)) => Exp::Var(name.clone()),
            None => return Err(ValidationError::UndefinedVariable(name)),
        },
        Exp::Unary(op @ (UnaryOp::PostfixIncrement | UnaryOp::PostfixDecrement), exp) => match *exp
        {
            exp @ Exp::Var(_) => Exp::Unary(op, Box::new(resolve_exp(exp, scope_manager)?)),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Unary(op, e) => Exp::Unary(op, Box::new(resolve_exp(*e, scope_manager)?)),
        Exp::Binary(op, e1, e2) => Exp::Binary(
            op,
            Box::new(resolve_exp(*e1, scope_manager)?),
            Box::new(resolve_exp(*e2, scope_manager)?),
        ),
        Exp::Conditional(cond, then, else_) => Exp::Conditional(
            Box::new(resolve_exp(*cond, scope_manager)?),
            Box::new(resolve_exp(*then, scope_manager)?),
            Box::new(resolve_exp(*else_, scope_manager)?),
        ),
        Exp::FuncCall(name, args) => {
            let Some((name, _)) = scope_manager.get(&name) else {
                return Err(ValidationError::UndefinedFunction(name));
            };

            let new_name = name.clone();
            let new_args = args
                .into_iter()
                .map(|arg| resolve_exp(arg, scope_manager))
                .collect::<Result<_, _>>()?;

            Exp::FuncCall(new_name, new_args)
        }
    };
    Ok(exp)
}
