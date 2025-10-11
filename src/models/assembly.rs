use derive_more::Display;

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Display)]
pub enum Instruction {
    #[display("    movl {}, {}", src, dst)]
    Mov { src: Operand, dst: Operand },
    #[display("    ret")]
    Ret,
}

#[derive(Debug, Display)]
pub enum Operand {
    #[display("${_0}")]
    Imm(i32),
    #[display("%eax")]
    Register,
}
