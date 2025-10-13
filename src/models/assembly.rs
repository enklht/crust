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
    Binary(BinaryOp, Operand, Operand),
    Cmp(Operand, Operand),
    Idiv(Operand),
    Shl(Operand),
    Shr(Operand),
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
    Cdq,
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

#[derive(Debug, Display)]
pub enum BinaryOp {
    #[display("addl")]
    Add,
    #[display("subl")]
    Subtract,
    #[display("imull")]
    Multiply,
    #[display("andl")]
    BitAnd,
    #[display("orl")]
    BitOr,
    #[display("xorl")]
    BitXor,
}

#[derive(Debug, Display, Clone)]
pub enum Operand {
    #[display("${_0}")]
    Imm(i32),
    #[display("{_0}")]
    Reg(Reg),
    #[display("<pseudo register will never be emit>")]
    Pseudo(String),
    #[display("{_0}(%rbp)")]
    Stack(isize),
}

impl Operand {
    pub fn onebyte(self) -> String {
        match self {
            Operand::Reg(reg) => reg.onebyte(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Display)]
pub enum CondCode {
    #[display("e")]
    E,
    #[display("ne")]
    NE,
    #[display("l")]
    L,
    #[display("le")]
    LE,
    #[display("g")]
    G,
    #[display("ge")]
    GE,
}

#[derive(Debug, Display, Clone)]
pub enum Reg {
    #[display("%eax")]
    AX,
    #[display("%ecx")]
    CX,
    #[display("%edx")]
    DX,
    #[display("%r10d")]
    R10,
    #[display("%r11d")]
    R11,
}

impl Reg {
    fn onebyte(self) -> String {
        match self {
            Reg::AX => "%al",
            Reg::CX => "%cl",
            Reg::DX => "%dl",
            Reg::R10 => "%r10b",
            Reg::R11 => "%r11b",
        }
        .to_string()
    }
}
