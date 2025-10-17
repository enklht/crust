#[derive(Debug)]
pub enum Program {
    Program(Function),
}

pub enum Function {
    Function(String, Vec<Instruction>),
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Function::Function(name, instructions) => {
                write!(
                    f,
                    "\n    name: {}\n    instructions:\n{}",
                    name,
                    instructions
                        .iter()
                        .fold(String::new(), |acc, i| acc + &format!("        {i:?}\n"))
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val),
    Copy(Val, Val),
    Jump(String),
    JumpIfZero(Val, String),
    JumpIfNotZero(Val, String),
    Label(String),
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
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}
