struct Program {
    function: Function,
}

struct Function {
    name: String,
    instruction: Vec<Instruction>,
}

enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

enum Operand {
    Imm(i32),
    Register,
}
