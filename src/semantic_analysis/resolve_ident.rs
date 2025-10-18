use super::ValidationError;
use super::ValidationResult;
use crate::models::ast::*;
use crate::utils::make_label;
use rustc_hash::FxHashMap;

#[derive(Clone)]
struct MapEntry {
    name: String,
    from_current_scope: bool,
    has_linkage: bool,
}

type IdentMap = FxHashMap<String, MapEntry>;

fn copy_map(ident_map: &IdentMap) -> IdentMap {
    ident_map
        .iter()
        .map(|(name, entry)| {
            let mut new_entry = entry.clone();
            new_entry.from_current_scope = false;
            (name.clone(), new_entry)
        })
        .collect()
}

pub fn resolve_program(program: Program) -> ValidationResult<Program> {
    let ident_map = &mut FxHashMap::default();
    let program = match program {
        Program::Program(func_decl) => Program::Program(
            func_decl
                .into_iter()
                .map(|func_decl| resolve_func_decl(func_decl, ident_map))
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(program)
}

fn resolve_block(block: Block, ident_map: &mut IdentMap) -> ValidationResult<Block> {
    block
        .into_iter()
        .map(|block_item| resolve_block_item(block_item, ident_map))
        .collect::<ValidationResult<_>>()
}

fn resolve_block_item(
    block_item: BlockItem,
    ident_map: &mut IdentMap,
) -> ValidationResult<BlockItem> {
    let block_item = match block_item {
        BlockItem::S(stmt) => BlockItem::S(resolve_stmt(stmt, ident_map)?),
        BlockItem::D(decl) => BlockItem::D(resolve_decl(decl, ident_map)?),
    };
    Ok(block_item)
}

fn resolve_decl(decl: Decl, ident_map: &mut IdentMap) -> ValidationResult<Decl> {
    let decl = match decl {
        Decl::VarDecl(var_decl) => Decl::VarDecl(resolve_var_decl(var_decl, ident_map)?),
        Decl::FuncDecl(func_decl) => {
            if let FuncDecl::FuncDecl(name, _, Some(_)) = func_decl {
                return Err(ValidationError::NestedFuncDef(name));
            }
            Decl::FuncDecl(resolve_func_decl(func_decl, ident_map)?)
        }
    };
    Ok(decl)
}

fn resolve_func_decl(func_decl: FuncDecl, ident_map: &mut IdentMap) -> ValidationResult<FuncDecl> {
    let func_decl = match func_decl {
        FuncDecl::FuncDecl(name, params, body) => {
            if let Some(entry) = ident_map.get(&name)
                && entry.from_current_scope
                && !entry.has_linkage
            {
                return Err(ValidationError::DuplicateFuncDecl(name));
            }

            ident_map.insert(
                name.clone(),
                MapEntry {
                    name: name.clone(),
                    from_current_scope: true,
                    has_linkage: true,
                },
            );

            let inner_ident_map = &mut copy_map(ident_map);
            let new_params = params
                .into_iter()
                .map(|param| resolve_ident(param, inner_ident_map))
                .collect::<Result<_, _>>()?;

            FuncDecl::FuncDecl(
                name,
                new_params,
                body.map(|block| resolve_block(block, inner_ident_map))
                    .transpose()?,
            )
        }
    };
    Ok(func_decl)
}

fn resolve_ident(name: String, ident_map: &mut IdentMap) -> ValidationResult<String> {
    if let Some(entry) = ident_map.get(&name)
        && entry.from_current_scope
    {
        return Err(ValidationError::DuplicateVarDecl(name));
    }

    let unique_name = make_label(&name);
    ident_map.insert(
        name,
        MapEntry {
            name: unique_name.clone(),
            from_current_scope: true,
            has_linkage: false,
        },
    );

    Ok(unique_name)
}

fn resolve_var_decl(var_decl: VarDecl, ident_map: &mut IdentMap) -> ValidationResult<VarDecl> {
    let var_decl = match var_decl {
        VarDecl::VarDecl(name, init) => {
            let unique_name = resolve_ident(name, ident_map)?;

            VarDecl::VarDecl(
                unique_name,
                init.map(|exp| resolve_exp(exp, ident_map)).transpose()?,
            )
        }
    };
    Ok(var_decl)
}

fn resolve_stmt(stmt: Stmt, ident_map: &mut IdentMap) -> ValidationResult<Stmt> {
    let stmt = match stmt {
        Stmt::Return(exp) => Stmt::Return(resolve_exp(exp, ident_map)?),
        Stmt::Expression(exp) => Stmt::Expression(resolve_exp(exp, ident_map)?),
        Stmt::If(cond, then, Some(else_)) => Stmt::If(
            resolve_exp(cond, ident_map)?,
            Box::new(resolve_stmt(*then, ident_map)?),
            Some(Box::new(resolve_stmt(*else_, ident_map)?)),
        ),
        Stmt::If(cond, then, None) => Stmt::If(
            resolve_exp(cond, ident_map)?,
            Box::new(resolve_stmt(*then, ident_map)?),
            None,
        ),
        Stmt::Labelled(label, stmt) => {
            Stmt::Labelled(label, Box::new(resolve_stmt(*stmt, ident_map)?))
        }
        Stmt::Goto(label) => Stmt::Goto(label),
        Stmt::Compound(block) => {
            let new_ident_map = &mut copy_map(ident_map);
            Stmt::Compound(resolve_block(block, new_ident_map)?)
        }
        Stmt::While(cond, body, label) => Stmt::While(
            resolve_exp(cond, ident_map)?,
            Box::new(resolve_stmt(*body, ident_map)?),
            label,
        ),
        Stmt::DoWhile(body, cond, label) => Stmt::DoWhile(
            Box::new(resolve_stmt(*body, ident_map)?),
            resolve_exp(cond, ident_map)?,
            label,
        ),
        Stmt::For(init, cond, post, body, label) => {
            let new_ident_map = &mut copy_map(ident_map);
            let init = resolve_for_init(init, new_ident_map)?;
            let cond = cond
                .map(|cond| resolve_exp(cond, new_ident_map))
                .transpose()?;
            let post = post
                .map(|post| resolve_exp(post, new_ident_map))
                .transpose()?;
            let body = Box::new(resolve_stmt(*body, new_ident_map)?);
            Stmt::For(init, cond, post, body, label)
        }
        Stmt::Switch(cond, body, label) => {
            let cond = resolve_exp(cond, ident_map)?;
            let body = Box::new(resolve_stmt(*body, ident_map)?);
            Stmt::Switch(cond, body, label)
        }
        // Break, Continue, Null
        other => other,
    };
    Ok(stmt)
}

fn resolve_for_init(init: ForInit, ident_map: &mut IdentMap) -> ValidationResult<ForInit> {
    let init = match init {
        ForInit::InitExp(exp) => ForInit::InitExp(resolve_exp(exp, ident_map)?),
        ForInit::InitDecl(var_decl) => ForInit::InitDecl(resolve_var_decl(var_decl, ident_map)?),
        ForInit::Null => ForInit::Null,
    };
    Ok(init)
}

fn resolve_exp(exp: Exp, ident_map: &mut IdentMap) -> ValidationResult<Exp> {
    let exp = match exp {
        Exp::Const(i) => Exp::Const(i),
        Exp::Assignment(left, right) => match *left {
            left @ Exp::Var(_) => Exp::Assignment(
                Box::new(resolve_exp(left, ident_map)?),
                Box::new(resolve_exp(*right, ident_map)?),
            ),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Var(name) => match ident_map.get(&name) {
            Some(entry) => Exp::Var(entry.name.clone()),
            None => return Err(ValidationError::UndefinedVariable(name)),
        },
        Exp::Unary(op @ (UnaryOp::PostfixIncrement | UnaryOp::PostfixDecrement), exp) => match *exp
        {
            exp @ Exp::Var(_) => Exp::Unary(op, Box::new(resolve_exp(exp, ident_map)?)),
            other => return Err(ValidationError::InvalidLvalue(other.clone())),
        },
        Exp::Unary(op, e) => Exp::Unary(op, Box::new(resolve_exp(*e, ident_map)?)),
        Exp::Binary(op, e1, e2) => Exp::Binary(
            op,
            Box::new(resolve_exp(*e1, ident_map)?),
            Box::new(resolve_exp(*e2, ident_map)?),
        ),
        Exp::Conditional(cond, then, else_) => Exp::Conditional(
            Box::new(resolve_exp(*cond, ident_map)?),
            Box::new(resolve_exp(*then, ident_map)?),
            Box::new(resolve_exp(*else_, ident_map)?),
        ),
        Exp::FuncCall(name, args) => {
            let Some(entry) = ident_map.get(&name) else {
                return Err(ValidationError::UndefinedFunction(name));
            };

            let new_name = entry.name.clone();
            let new_args = args
                .into_iter()
                .map(|arg| resolve_exp(arg, ident_map))
                .collect::<Result<_, _>>()?;

            Exp::FuncCall(new_name, new_args)
        }
    };
    Ok(exp)
}
