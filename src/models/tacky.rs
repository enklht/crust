#[derive(Debug)]
pub enum Program {
    Program(Function),
}

#[derive(Debug)]
pub enum Function {
    Function(String, Vec<Instruction>),
}

#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val),
}

#[derive(Debug, Clone)]
pub enum Val {
    Const(i32),
    Var(String),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negate,
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
}
