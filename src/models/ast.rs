use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub enum Program {
    Program(Vec<FuncDecl>),
}

#[derive(Debug, Clone)]
pub enum Decl {
    VarDecl(VarDecl),
    FuncDecl(FuncDecl),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(String, Option<Exp>),
}

#[derive(Debug, Clone)]
pub enum FuncDecl {
    FuncDecl(String, Vec<String>, Option<Block>),
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    Labelled(Label, Box<Stmt>),
    Goto(String),
    Compound(Block),
    Break(Option<String>),
    Continue(Option<String>),
    While(Exp, Box<Stmt>, Option<String>),
    DoWhile(Box<Stmt>, Exp, Option<String>),
    For(ForInit, Option<Exp>, Option<Exp>, Box<Stmt>, Option<String>),
    Switch(Exp, Box<Stmt>, Option<String>),
    SwitchResolved(
        Exp,
        Box<Stmt>,
        Option<String>,
        FxHashMap<i32, String>,
        Option<String>,
    ),
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
