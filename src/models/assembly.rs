use derive_more::Display;

#[derive(Debug)]
pub enum Program {
    Program(Function),
}

#[derive(Debug)]
pub enum Function {
    Function {
        name: String,
        instructions: Vec<Instruction>,
    },
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
