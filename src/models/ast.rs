#[derive(Debug)]
pub enum Program {
    FuncDef(FuncDef),
}

#[derive(Debug)]
pub enum FuncDef {
    Function(String, Vec<BlockItem>),
}

#[derive(Debug)]
pub enum BlockItem {
    S(Stmt),
    D(Decl),
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    Expression(Expr),
    Null,
}

#[derive(Debug)]
pub enum Decl {
    Decl(String, Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Const(i32),
    Var(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Complement,
    Negate,
    Not,
    PrefixIncrement,
    PrefixDecrement,
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
