use rustc_hash::FxHashMap;

pub trait AstEx {
    type Label: std::fmt::Debug + Clone;
    type Switch: std::fmt::Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Plain;
impl AstEx for Plain {
    type Label = ();
    type Switch = ();
}

#[derive(Debug, Clone)]
pub struct Labelled;
impl AstEx for Labelled {
    type Label = String;
    type Switch = ();
}

#[derive(Debug, Clone)]
pub struct SwitchResolved;
impl AstEx for SwitchResolved {
    type Label = String;
    type Switch = (FxHashMap<i32, String>, Option<String>);
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
    FuncDecl(String, Vec<String>, Option<Block<Ex>>, Option<StorageClass>),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(String, Option<Exp>, Option<StorageClass>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClass {
    Static,
    Extern,
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
    Switch(Exp, Box<Stmt<Ex>>, Ex::Label, Ex::Switch),
    Null,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Const(i32),
    Var(String),
    Unary(UnaryOp, Box<Exp>),
    Binary(BinaryOp, Box<Exp>, Box<Exp>),
    Assignment(Box<Exp>, Box<Exp>),
    Conditional(Box<Exp>, Box<Exp>, Box<Exp>),
    FuncCall(String, Vec<Exp>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    BitNot,
    Not,
    PostfixIncrement,
    PostfixDecrement,
}

#[derive(Debug, Clone)]
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
