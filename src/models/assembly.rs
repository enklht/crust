use derive_more::Display;

use crate::models::symbol::StaticInit;

#[derive(Debug)]
pub enum Program {
    Program(Vec<TopLevel>),
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum AssemblyType {
    #[display("b")]
    Byte,
    #[display("l")]
    Longword,
    #[display("q")]
    Quadword,
}

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
    #[display("neg")]
    Neg,
    #[display("not")]
    Not,
}

#[derive(Debug, Display)]
pub enum BinaryOp {
    #[display("add")]
    Add,
    #[display("sub")]
    Subtract,
    #[display("imul")]
    Multiply,
    #[display("and")]
    BitAnd,
    #[display("or")]
    BitOr,
    #[display("xor")]
    BitXor,
    #[display("sal")]
    LShift,
    #[display("sar")]
    RShift,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i64),
    Reg(Reg),
    Pseudo(String),
    Stack(isize),
    Data(String),
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

pub fn fmt<'a>(op: &'a Operand, typ: AssemblyType) -> OperandFmt<'a> {
    OperandFmt { op, typ }
}

pub struct OperandFmt<'a> {
    op: &'a Operand,
    typ: AssemblyType,
}

impl<'a> std::fmt::Display for OperandFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            AssemblyType::Byte => self.op.one_byte(f),
            AssemblyType::Longword => self.op.four_byte(f),
            AssemblyType::Quadword => self.op.eight_byte(f),
        }
    }
}

impl Operand {
    pub fn one_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => reg.one_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Stack(offset) => write!(f, "{}(%rbp)", offset),
            Operand::Data(label) => write!(f, "{}(%rip)", label),
            Operand::Pseudo(_) => panic!("pseudo register will never be emit"),
        }
    }

    pub fn four_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => reg.four_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Stack(offset) => write!(f, "{}(%rbp)", offset),
            Operand::Data(label) => write!(f, "{}(%rip)", label),
            Operand::Pseudo(_) => panic!("pseudo register will never be emit"),
        }
    }

    pub fn eight_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => reg.eight_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Stack(offset) => write!(f, "{}(%rbp)", offset),
            Operand::Data(label) => write!(f, "{}(%rip)", label),
            Operand::Pseudo(_) => panic!("pseudo register will never be emit"),
        }
    }
}

impl Reg {
    pub fn one_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::AX => write!(f, "%al"),
            Reg::CX => write!(f, "%cl"),
            Reg::DX => write!(f, "%dl"),
            Reg::DI => write!(f, "%dil"),
            Reg::SI => write!(f, "%sil"),
            Reg::R8 => write!(f, "%r8b"),
            Reg::R9 => write!(f, "%r9b"),
            Reg::R10 => write!(f, "%r10b"),
            Reg::R11 => write!(f, "%r11b"),
            Reg::SP => write!(f, "%spl"),
        }
    }

    fn four_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::AX => write!(f, "%eax"),
            Reg::CX => write!(f, "%ecx"),
            Reg::DX => write!(f, "%edx"),
            Reg::DI => write!(f, "%edi"),
            Reg::SI => write!(f, "%esi"),
            Reg::R8 => write!(f, "%r8d"),
            Reg::R9 => write!(f, "%r9d"),
            Reg::R10 => write!(f, "%r10d"),
            Reg::R11 => write!(f, "%r11d"),
            Reg::SP => write!(f, "%esp"),
        }
    }

    fn eight_byte(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::AX => write!(f, "%rax"),
            Reg::CX => write!(f, "%rcx"),
            Reg::DX => write!(f, "%rdx"),
            Reg::DI => write!(f, "%rdi"),
            Reg::SI => write!(f, "%rsi"),
            Reg::R8 => write!(f, "%r8"),
            Reg::R9 => write!(f, "%r9"),
            Reg::R10 => write!(f, "%r10"),
            Reg::R11 => write!(f, "%r11"),
            Reg::SP => write!(f, "%rsp"),
        }
    }
}

impl std::fmt::Debug for TopLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TopLevel::Function(name, global, instructions, _) => {
                writeln!(f, "\nname: {name}")?;
                writeln!(f, "global: {global}")?;
                write!(
                    f,
                    "instructions: {:?}",
                    crate::utils::PrettyVec(instructions)
                )
            }
            TopLevel::StaticVar(name, global, _, init) => {
                writeln!(f, "\nstatic {name} := {init:?}")?;
                write!(f, "global: {global}")
            }
        }
    }
}
