pub use super::constant::Const;
pub use super::operator::*;
use super::typ::Type;
use crate::utils::Ident;
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
    FuncDecl(Ident, Vec<Ident>, Option<Block>, Type, StorageClass),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(Ident, Option<Exp>, Type, StorageClass),
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
    Ident(Ident),
    Case(Exp),
    Default,
}

type BlockLabel = Option<Ident>;
type SwitchInfo = Option<(FxHashMap<Const, Ident>, Option<Ident>)>;

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    Labelled(Label, Box<Stmt>),
    Goto(Ident),
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
    Var(Ident, Option<Type>),
    Cast(Type, Box<Exp>),
    Unary(UnaryOp, Box<Exp>, Option<Type>),
    Binary(BinaryOp, Box<Exp>, Box<Exp>, Option<Type>),
    Assignment(Box<Exp>, Box<Exp>, Option<Type>),
    Conditional(Box<Exp>, Box<Exp>, Box<Exp>, Option<Type>),
    FuncCall(Ident, Vec<Exp>, Option<Type>),
    Dereference(Box<Exp>, Option<Type>),
    AddressOf(Box<Exp>, Option<Type>),
}

impl Exp {
    pub fn typ(&self) -> &Type {
        use Exp::*;
        match self {
            Const(c) => c.typ(),
            Var(_, Some(typ)) => typ,
            Cast(typ, _) => typ,
            Unary(_, _, Some(typ)) => typ,
            Binary(_, _, _, Some(typ)) => typ,
            Assignment(_, _, Some(typ)) => typ,
            Conditional(_, _, _, Some(typ)) => typ,
            FuncCall(_, _, Some(typ)) => typ,
            _ => unreachable!("the type is not assigned"),
        }
    }

    pub fn convert_to(&mut self, typ: &Type) {
        if self.typ() != typ {
            *self = Exp::Cast(typ.clone(), Box::new(self.clone()))
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
