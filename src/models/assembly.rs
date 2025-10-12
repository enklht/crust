use derive_more::Display;

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
    Mov(Operand, Operand),
    Unary(UnaryOp, Operand),
    AllocateStack(usize),
    Ret,
}

#[derive(Debug, Display)]
pub enum UnaryOp {
    #[display("negl")]
    Neg,
    #[display("notl")]
    Not,
}

#[derive(Debug, Display, Clone)]
pub enum Operand {
    #[display("${_0}")]
    Imm(i64),
    #[display("{_0}")]
    Reg(Reg),
    #[display("<pseudo register will never be emit>")]
    Pseudo(String),
    #[display("{_0}(%rbp)")]
    Stack(isize),
}

#[derive(Debug, Display, Clone)]
pub enum Reg {
    #[display("%eax")]
    AX,
    #[display("%r10d")]
    R10,
}
