use crate::models::typ::Type;
use rustc_hash::FxHashMap;

pub trait AstEx {
    type Label: std::fmt::Debug + Clone;
    type SwitchInfo: std::fmt::Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Plain;
impl AstEx for Plain {
    type Label = ();
    type SwitchInfo = ();
}

#[derive(Debug, Clone)]
pub struct Labelled;
impl AstEx for Labelled {
    type Label = String;
    type SwitchInfo = ();
}

#[derive(Debug, Clone)]
pub struct SwitchResolved;
impl AstEx for SwitchResolved {
    type Label = String;
    type SwitchInfo = (FxHashMap<Const, String>, Option<String>);
}

#[derive(Debug, Clone)]
pub enum Program<Ex: AstEx> {
    Program(Vec<Decl<Ex>>),
}

#[derive(Debug, Clone)]
pub enum Decl<Ex: AstEx> {
    FuncDecl(FuncDecl<Ex>),
    VarDecl(VarDecl),
}

#[derive(Debug, Clone)]
pub enum FuncDecl<Ex: AstEx> {
    FuncDecl(String, Vec<String>, Option<Block<Ex>>, Type, StorageClass),
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

pub type Block<Ex> = Vec<BlockItem<Ex>>;

#[derive(Debug, Clone)]
pub enum BlockItem<Ex: AstEx> {
    S(Stmt<Ex>),
    D(Decl<Ex>),
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

#[derive(Debug, Clone)]
pub enum Stmt<Ex: AstEx> {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt<Ex>>, Option<Box<Stmt<Ex>>>),
    Labelled(Label, Box<Stmt<Ex>>),
    Goto(String),
    Compound(Block<Ex>),
    Break(Ex::Label),
    Continue(Ex::Label),
    While(Exp, Box<Stmt<Ex>>, Ex::Label),
    DoWhile(Box<Stmt<Ex>>, Exp, Ex::Label),
    For(ForInit, Option<Exp>, Option<Exp>, Box<Stmt<Ex>>, Ex::Label),
    Switch(Exp, Box<Stmt<Ex>>, Ex::Label, Ex::SwitchInfo),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Negate,
    BitNot,
    Not,
    PostfixIncrement,
    PostfixDecrement,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Const {
    Int(i32),
    Long(i64),
}
