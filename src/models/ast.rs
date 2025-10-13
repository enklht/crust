#[derive(Debug)]
pub enum Program {
    FunctionDefinition(FunctionDefinition),
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String, Vec<BlockItem>),
}

#[derive(Debug)]
pub enum BlockItem {
    S(Stmt),
    D(Declaration),
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    Expression(Expr),
    Null,
}

#[derive(Debug)]
pub enum Declaration {
    Declaration(String, Option<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    Const(i32),
    Var(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negate,
    Not,
}

#[derive(Debug)]
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
