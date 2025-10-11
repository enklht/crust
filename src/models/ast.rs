#[derive(Debug)]
pub enum Program {
    Function(Function),
}

#[derive(Debug)]
pub enum Function {
    Function { name: Ident, body: Stmt },
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Int(Int),
}

#[derive(Debug)]
pub enum Ident {
    Ident(String),
}

#[derive(Debug)]
pub enum Int {
    Const(i32),
}
