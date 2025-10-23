use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    BitNot,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnaryOp::*;
        match self {
            Negate => write!(f, "neg"),
            BitNot => write!(f, "not"),
            _ => panic!("display is not implemented for this variant"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    Divide,
    Remainder,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryOp::*;
        match self {
            Add => write!(f, "add"),
            Subtract => write!(f, "sub"),
            Multiply => write!(f, "imul"),
            BitAnd => write!(f, "and"),
            BitOr => write!(f, "or"),
            BitXor => write!(f, "xor"),
            _ => panic!("display is not implemented for this variant"),
        }
    }
}

#[derive(Debug)]
pub enum ShiftOp {
    Sal,
    Sar,
    Shl,
    Shr,
}

impl fmt::Display for ShiftOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ShiftOp::*;
        match self {
            Sal => write!(f, "sal"),
            Sar => write!(f, "sar"),
            Shl => write!(f, "shl"),
            Shr => write!(f, "shr"),
        }
    }
}
