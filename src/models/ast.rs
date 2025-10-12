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
    Const(i64),
    Unary(UnaryOp, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negate,
}
