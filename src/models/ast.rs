#[derive(Debug)]
pub enum Program {
    Function(Function),
}

#[derive(Debug)]
pub enum Function {
    Function { name: String, body: Stmt },
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Const(i32),
    Unary(UnaryOp, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negate,
}
