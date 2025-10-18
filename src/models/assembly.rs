use derive_more::Display;

pub trait AsmExtra {
    type FuncDef: std::fmt::Debug;
}

#[derive(Debug)]
pub struct Plain;
impl AsmExtra for Plain {
    type FuncDef = ();
}

#[derive(Debug)]
pub struct LastOffset;
impl AsmExtra for LastOffset {
    type FuncDef = usize;
}

#[derive(Debug)]
pub enum Program<Extra: AsmExtra> {
    Program(Vec<FuncDef<Extra>>),
}

#[derive(Debug)]
pub enum FuncDef<Extra: AsmExtra> {
    Function(String, Vec<Instruction>, Extra::FuncDef),
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
    DeallocateStack(usize),
    Push(Operand),
    Call(String),
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
    #[display("{}", _0.four_byte())]
    Reg(Reg),
    #[display("<pseudo register will never be emit>")]
    Pseudo(String),
    #[display("{_0}(%rbp)")]
    Stack(isize),
}

impl Operand {
    pub fn one_byte(self) -> String {
        match self {
            Operand::Reg(reg) => reg.one_byte(),
            other => other.to_string(),
        }
    }

    pub fn eight_byte(self) -> String {
        match self {
            Operand::Reg(reg) => reg.eight_byte(),
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

#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
}

impl Reg {
    fn eight_byte(&self) -> String {
        match self {
            Reg::AX => "%rax",
            Reg::CX => "%rcx",
            Reg::DX => "%rdx",
            Reg::DI => "%rdi",
            Reg::SI => "%rsi",
            Reg::R8 => "%r8",
            Reg::R9 => "%r9",
            Reg::R10 => "%r10",
            Reg::R11 => "%r11",
        }
        .to_string()
    }

    fn four_byte(&self) -> String {
        match self {
            Reg::AX => "%eax",
            Reg::CX => "%ecx",
            Reg::DX => "%edx",
            Reg::DI => "%edi",
            Reg::SI => "%esi",
            Reg::R8 => "%r8d",
            Reg::R9 => "%r9d",
            Reg::R10 => "%r10d",
            Reg::R11 => "%r11d",
        }
        .to_string()
    }

    fn one_byte(&self) -> String {
        match self {
            Reg::AX => "%al",
            Reg::CX => "%cl",
            Reg::DX => "%dl",
            Reg::DI => "%dil",
            Reg::SI => "%sil",
            Reg::R8 => "%r8b",
            Reg::R9 => "%r9b",
            Reg::R10 => "%r10b",
            Reg::R11 => "%r11b",
        }
        .to_string()
    }
}
