use crate::models::asm_symbol::AsmSymbol;
use crate::models::asm_symbol::AsmSymbols;
use crate::models::symbol::StaticInit;
use crate::utils::Ident;
use derive_more::Display;
use std::fmt;

#[derive(Debug)]
pub enum Program {
    Program(Vec<TopLevel>),
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum AsmType {
    #[display("")]
    Byte,
    #[display("l")]
    Longword,
    #[display("q")]
    Quadword,
    #[display("sd")]
    Double,
}

pub enum TopLevel {
    Function(Ident, bool, Vec<Instruction>, Option<usize>),
    StaticVar(Ident, bool, usize, StaticInit),
    StaticConst(Ident, usize, StaticInit),
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand, AsmType),
    Movsx(Operand, Operand),
    MovZeroExtend(Operand, Operand),
    Lea(Operand, Operand),
    Cvttsd2si(Operand, Operand, AsmType),
    Cvtsi2sd(Operand, Operand, AsmType),
    Unary(UnaryOp, Operand, AsmType),
    Binary(BinaryOp, Operand, Operand, AsmType),
    Cmp(Operand, Operand, AsmType),
    Idiv(Operand, AsmType),
    Div(Operand, AsmType),
    Cdq(AsmType),
    Jmp(Ident),
    JmpCC(CondCode, Ident),
    SetCC(CondCode, Operand),
    Label(Ident),
    Push(Operand),
    Call(Ident),
    Ret,
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Imm(u64),
    Reg(Reg),
    Pseudo(Ident),
    Memory(Reg, isize),
    Data(Ident),
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
    #[display("a")]
    A,
    #[display("ae")]
    AE,
    #[display("b")]
    B,
    #[display("be")]
    BE,
    #[display("p")]
    P,
    #[display("np")]
    NP,
}

#[derive(Debug, Clone, Copy)]
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
    BP,
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM14,
    XMM15,
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum UnaryOp {
    #[display("neg")]
    Negate,
    #[display("not")]
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum BinaryOp {
    #[display("add")]
    Add,
    #[display("sub")]
    Subtract,
    #[display("imul")]
    Multiply,
    #[display("div")]
    DivideDouble,
    #[display("and")]
    BitAnd,
    #[display("or")]
    BitOr,
    #[display("xor")]
    BitXor,
    #[display("sal")]
    Sal,
    #[display("sar")]
    Sar,
    #[display("shl")]
    Shl,
    #[display("shr")]
    Shr,
}

pub fn fmt<'a>(op: &'a Operand, typ: AsmType, symbols: &'a AsmSymbols) -> OperandFmt<'a> {
    OperandFmt { op, typ, symbols }
}

pub struct OperandFmt<'a> {
    op: &'a Operand,
    typ: AsmType,
    symbols: &'a AsmSymbols,
}

impl<'a> fmt::Display for OperandFmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.typ {
            AsmType::Byte => self.op.one_byte(f, self.symbols),
            AsmType::Longword => self.op.four_byte(f, self.symbols),
            AsmType::Quadword => self.op.eight_byte(f, self.symbols),
            AsmType::Double => self.op.eight_byte(f, self.symbols),
        }
    }
}

impl Operand {
    pub fn one_byte(&self, f: &mut fmt::Formatter<'_>, symbols: &AsmSymbols) -> fmt::Result {
        match self {
            Operand::Reg(reg) => reg.one_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Memory(reg, offset) => {
                write!(f, "{}(", offset)?;
                reg.eight_byte(f)?;
                write!(f, ")")
            }
            Operand::Data(label) => match symbols.get(label).unwrap() {
                AsmSymbol::ObjEntry(_, _, true) => {
                    write!(f, ".L{}(%rip)", label)
                }
                _ => {
                    write!(f, "{}(%rip)", label)
                }
            },
            Operand::Pseudo(_) => unreachable!("pseudo register will never be emit"),
        }
    }

    pub fn four_byte(&self, f: &mut fmt::Formatter<'_>, symbols: &AsmSymbols) -> fmt::Result {
        match self {
            Operand::Reg(reg) => reg.four_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Memory(reg, offset) => {
                write!(f, "{}(", offset)?;
                reg.eight_byte(f)?;
                write!(f, ")")
            }
            Operand::Data(label) => match symbols.get(label).unwrap() {
                AsmSymbol::ObjEntry(_, _, true) => {
                    write!(f, ".L{}(%rip)", label)
                }
                _ => {
                    write!(f, "{}(%rip)", label)
                }
            },
            Operand::Pseudo(_) => unreachable!("pseudo register will never be emit"),
        }
    }

    pub fn eight_byte(&self, f: &mut fmt::Formatter<'_>, symbols: &AsmSymbols) -> fmt::Result {
        match self {
            Operand::Reg(reg) => reg.eight_byte(f),
            Operand::Imm(val) => write!(f, "${}", val),
            Operand::Memory(reg, offset) => {
                write!(f, "{}(", offset)?;
                reg.eight_byte(f)?;
                write!(f, ")")
            }
            Operand::Data(label) => match symbols.get(label).unwrap() {
                AsmSymbol::ObjEntry(_, _, true) => {
                    write!(f, ".L{}(%rip)", label)
                }
                _ => {
                    write!(f, "{}(%rip)", label)
                }
            },
            Operand::Pseudo(_) => unreachable!("pseudo register will never be emit"),
        }
    }
}

impl Reg {
    pub fn one_byte(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            _ => unreachable!(),
        }
    }

    fn four_byte(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            _ => unreachable!(),
        }
    }

    fn eight_byte(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            Reg::BP => write!(f, "%rbp"),
            Reg::XMM0 => write!(f, "%xmm0"),
            Reg::XMM1 => write!(f, "%xmm1"),
            Reg::XMM2 => write!(f, "%xmm2"),
            Reg::XMM3 => write!(f, "%xmm3"),
            Reg::XMM4 => write!(f, "%xmm4"),
            Reg::XMM5 => write!(f, "%xmm5"),
            Reg::XMM6 => write!(f, "%xmm6"),
            Reg::XMM7 => write!(f, "%xmm7"),
            Reg::XMM14 => write!(f, "%xmm14"),
            Reg::XMM15 => write!(f, "%xmm15"),
        }
    }
}

impl fmt::Debug for TopLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                writeln!(f, "\nstatic var {name} := {init:?}")?;
                write!(f, "global: {global}")
            }
            TopLevel::StaticConst(name, alignment, init) => {
                writeln!(f, "\nstatic const {name} := {init:?}")?;
                write!(f, "alignment: {alignment}")
            }
        }
    }
}
