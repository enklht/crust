use derive_more::Display;

use crate::models::symbol::StaticInit;

#[derive(Debug)]
pub enum Program {
    Program(Vec<TopLevel>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssemblyType {
    Longword,
    Quadword,
}

#[derive(Debug)]
pub enum TopLevel {
    Function(String, bool, Vec<Instruction>, Option<usize>),
    StaticVar(String, bool, usize, StaticInit),
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand, AssemblyType),
    Movsx(Operand, Operand),
    Unary(UnaryOp, Operand, AssemblyType),
    Binary(BinaryOp, Operand, Operand, AssemblyType),
    Cmp(Operand, Operand, AssemblyType),
    Idiv(Operand, AssemblyType),
    Cdq(AssemblyType),
    Shl(Operand, AssemblyType),
    Shr(Operand, AssemblyType),
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
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
    Imm(i64),
    #[display("{}", _0.four_byte())]
    Reg(Reg),
    #[display("<pseudo register will never be emit>")]
    Pseudo(String),
    #[display("{_0}(%rbp)")]
    Stack(isize),
    #[display("{_0}(%rip)")]
    Data(String),
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
    SP,
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
            Reg::SP => "%rsp",
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
            _ => panic!(),
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
            _ => panic!(),
        }
        .to_string()
    }
}
