use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,
    BitNot,
    LogicalNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum BinaryOp {
    #[display("+")]
    Add,
    #[display("-")]
    Subtract,
    #[display("*")]
    Multiply,
    #[display("&")]
    BitAnd,
    #[display("|")]
    BitOr,
    #[display("^")]
    BitXor,
    #[display("<<")]
    LShift,
    #[display(">>")]
    RShift,
    #[display("/")]
    Divide,
    #[display("%")]
    Remainder,
    #[display("&&")]
    LogicalAnd,
    #[display("||")]
    LogicalOr,
    #[display("==")]
    Equal,
    #[display("!=")]
    NotEqual,
    #[display("<")]
    LessThan,
    #[display("<=")]
    LessOrEqual,
    #[display(">")]
    GreaterThan,
    #[display(">=")]
    GreaterOrEqual,
}
