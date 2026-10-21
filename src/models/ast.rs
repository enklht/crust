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
    VarDecl(Ident, Option<Initializer>, Type, StorageClass),
}

#[derive(Debug, Clone)]
pub enum Initializer {
    Single(Exp),
    Compound(Vec<Initializer>),
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
pub struct Exp {
    pub kind: ExpKind,
    pub typ: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpKind {
    Const(Const),
    Var(Ident),
    Cast(Type, Box<Exp>),
    Unary(UnaryOp, Box<Exp>),
    Binary(BinaryOp, Box<Exp>, Box<Exp>),
    Assignment(Box<Exp>, Box<Exp>),
    Conditional(Box<Exp>, Box<Exp>, Box<Exp>),
    FuncCall(Ident, Vec<Exp>),
    Dereference(Box<Exp>),
    AddressOf(Box<Exp>),
    Update(BinaryOp, Box<Exp>, Box<Exp>, bool),
    Sequence(Vec<Exp>),
    Subscript(Box<Exp>, Box<Exp>),
}

impl Exp {
    pub fn new(kind: ExpKind) -> Self {
        Self { kind, typ: None }
    }

    pub fn typ(&self) -> &Type {
        self.typ.as_ref().expect("the type is not assigned")
    }

    pub fn is_null_pointer_constant(&self) -> bool {
        use Const::*;
        matches!(
            self.kind,
            ExpKind::Const(Int(0) | Long(0) | UInt(0) | ULong(0))
        )
    }
    pub fn is_lvalue(&self) -> bool {
        matches!(
            self.kind,
            ExpKind::Var(_) | ExpKind::Dereference(_) | ExpKind::Subscript(_, _)
        )
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
