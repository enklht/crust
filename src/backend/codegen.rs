use crate::models::assembly as asm;
use crate::models::tacky;

pub fn gen_program(program: tacky::Program) -> asm::Program {
    match program {
        tacky::Program::Program(function) => asm::Program::Program(gen_function(function)),
    }
}

fn gen_function(function: tacky::Function) -> asm::Function {
    match function {
        tacky::Function::Function(name, body) => {
            asm::Function::Function(name, gen_instructions(body))
        }
    }
}

fn gen_instructions(instructions: Vec<tacky::Instruction>) -> Vec<asm::Instruction> {
    let mut assembly_instructions = Vec::new();
    for instruction in instructions {
        match instruction {
            tacky::Instruction::Return(val) => {
                assembly_instructions.push(asm::Instruction::Mov(
                    gen_val(val),
                    asm::Operand::Reg(asm::Reg::AX),
                ));
                assembly_instructions.push(asm::Instruction::Ret)
            }
            tacky::Instruction::Unary(op, src, dst) => {
                let dst = gen_val(dst);
                assembly_instructions.push(asm::Instruction::Mov(gen_val(src), dst.clone()));
                assembly_instructions.push(asm::Instruction::Unary(gen_op(op), dst))
            }
            _ => todo!(),
        }
    }
    assembly_instructions
}

fn gen_val(val: tacky::Val) -> asm::Operand {
    match val {
        tacky::Val::Const(i) => asm::Operand::Imm(i),
        tacky::Val::Var(ident) => asm::Operand::Pseudo(ident),
    }
}

fn gen_op(op: tacky::UnaryOp) -> asm::UnaryOp {
    match op {
        tacky::UnaryOp::Complement => asm::UnaryOp::Not,
        tacky::UnaryOp::Negate => asm::UnaryOp::Neg,
    }
}
