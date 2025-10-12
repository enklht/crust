#[derive(Debug)]
pub enum Program {
    Function(Function),
}

#[derive(Debug)]
pub enum Function {
    Function(String, Stmt),
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Const(i32),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
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
