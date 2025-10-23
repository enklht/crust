pub use super::constant::Const;
pub use super::operator::*;
use super::typ::Type;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub enum Program {
    Program(Vec<Decl>),
}

#[derive(Debug, Clone)]
pub enum Decl {
    FuncDecl(FuncDecl),
    VarDecl(VarDecl),
}

#[derive(Clone)]
pub enum FuncDecl {
    FuncDecl(String, Vec<String>, Option<Block>, Type, StorageClass),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(String, Option<Exp>, Type, StorageClass),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClass {
    Static,
    Extern,
    None,
}

pub type Block = Vec<BlockItem>;

#[derive(Debug, Clone)]
pub enum BlockItem {
    S(Stmt),
    D(Decl),
}

#[derive(Debug, Clone)]
pub enum ForInit {
    InitDecl(VarDecl),
    InitExp(Exp),
    Null,
}

#[derive(Debug, Clone)]
pub enum Label {
    Ident(String),
    Case(Exp),
    Default,
}

type BlockLabel = Option<String>;
type SwitchInfo = Option<(FxHashMap<Const, String>, Option<String>)>;

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    Labelled(Label, Box<Stmt>),
    Goto(String),
    Compound(Block),
    Break(BlockLabel),
    Continue(BlockLabel),
    While(Exp, Box<Stmt>, BlockLabel),
    DoWhile(Box<Stmt>, Exp, BlockLabel),
    For(ForInit, Option<Exp>, Option<Exp>, Box<Stmt>, BlockLabel),
    Switch(Exp, Box<Stmt>, BlockLabel, SwitchInfo),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Exp {
    Const(Const),
    Var(String, Option<Type>),
    Cast(Type, Box<Exp>),
    Unary(UnaryOp, Box<Exp>, Option<Type>),
    Binary(BinaryOp, Box<Exp>, Box<Exp>, Option<Type>),
    Assignment(Box<Exp>, Box<Exp>, Option<Type>),
    Conditional(Box<Exp>, Box<Exp>, Box<Exp>, Option<Type>),
    FuncCall(String, Vec<Exp>, Option<Type>),
}

impl Exp {
    pub fn typ(&self) -> &Type {
        use Exp as E;
        match self {
            E::Const(Const::Int(_)) => &Type::Int,
            E::Const(Const::Long(_)) => &Type::Long,
            E::Var(_, Some(typ)) => typ,
            E::Cast(typ, _) => typ,
            E::Unary(_, _, Some(typ)) => typ,
            E::Binary(_, _, _, Some(typ)) => typ,
            E::Assignment(_, _, Some(typ)) => typ,
            E::Conditional(_, _, _, Some(typ)) => typ,
            E::FuncCall(_, _, Some(typ)) => typ,
            _ => panic!("the type is not assigned"),
        }
    }
}

impl std::fmt::Debug for FuncDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuncDecl::FuncDecl(name, params, body, typ, storage_class) => {
                writeln!(f, "name: {name}")?;
                writeln!(f, "type: {typ:?}")?;
                writeln!(f, "storage_class: {storage_class:?}")?;
                writeln!(f, "params: {params:?}")?;
                if let Some(body) = body {
                    write!(f, "instructions: {:?}", crate::utils::PrettyVec(body))
                } else {
                    Ok(())
                }
            }
        }
    }
}
