#[derive(Debug, Clone)]
pub enum Program {
    FuncDef(FuncDef),
}

#[derive(Debug, Clone)]
pub enum FuncDef {
    Function(String, Block),
}

pub type Block = Vec<BlockItem>;

#[derive(Debug, Clone)]
pub enum BlockItem {
    S(Stmt),
    D(Decl),
}

#[derive(Debug, Clone)]
pub enum Decl {
    Decl(String, Option<Exp>),
}

#[derive(Debug, Clone)]
pub enum ForInit {
    InitDecl(Decl),
    InitExp(Option<Exp>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    Labelled(String, Box<Stmt>),
    Goto(String),
    Compound(Block),
    Break(Option<String>),
    Continue(Option<String>),
    While(Exp, Box<Stmt>, Option<String>),
    DoWhile(Box<Stmt>, Exp, Option<String>),
    For(ForInit, Option<Exp>, Option<Exp>, Box<Stmt>, Option<String>),
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
