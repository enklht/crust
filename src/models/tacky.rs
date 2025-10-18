#[derive(Debug)]
pub enum Program {
    Program(Vec<FuncDef>),
}

pub enum FuncDef {
    FuncDef(String, Vec<String>, Vec<Instruction>),
}

impl std::fmt::Debug for FuncDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FuncDef::FuncDef(name, params, instructions) => {
                write!(
                    f,
                    "\n    name: {}\n    params: {:?}\n    instructions:\n{}",
                    name,
                    params,
                    instructions
                        .iter()
                        .fold(String::new(), |acc, i| acc + &format!("        {i:?},\n"))
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
    FuncCall(String, Vec<Val>, Val),
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
