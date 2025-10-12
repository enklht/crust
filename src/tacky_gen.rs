use crate::models::ast;
use crate::models::tacky;
use std::sync::atomic::{AtomicUsize, Ordering};

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn make_temporary() -> String {
    let i = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tmp.{}", i)
}

pub fn convert_program(program: ast::Program) -> tacky::Program {
    match program {
        ast::Program::Function(function) => tacky::Program::Program(convert_function(function)),
    }
}

fn convert_function(function: ast::Function) -> tacky::Function {
    match function {
        ast::Function::Function(name, body) => {
            let mut instructions = Vec::new();
            emit_stmt(body, &mut instructions);
            tacky::Function::Function(name, instructions)
        }
    }
}

fn emit_stmt(stmt: ast::Stmt, instructions: &mut Vec<tacky::Instruction>) {
    match stmt {
        ast::Stmt::Return(expr) => {
            let tacky_expr = emit_expr(expr, instructions);
            instructions.push(tacky::Instruction::Return(tacky_expr))
        }
    }
}

fn emit_expr(expr: ast::Expr, instructions: &mut Vec<tacky::Instruction>) -> tacky::Val {
    match expr {
        ast::Expr::Const(i) => tacky::Val::Const(i),
        ast::Expr::Unary(op, inner) => {
            let src = emit_expr(*inner, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_unop(op);
            instructions.push(tacky::Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        ast::Expr::Binary(op, e1, e2) => {
            let v1 = emit_expr(*e1, instructions);
            let v2 = emit_expr(*e2, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_binop(op);
            instructions.push(tacky::Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            dst
        }
    }
}

fn convert_unop(op: ast::UnaryOp) -> tacky::UnaryOp {
    match op {
        ast::UnaryOp::Complement => tacky::UnaryOp::Complement,
        ast::UnaryOp::Negate => tacky::UnaryOp::Negate,
    }
}

fn convert_binop(op: ast::BinaryOp) -> tacky::BinaryOp {
    match op {
        ast::BinaryOp::Add => tacky::BinaryOp::Add,
        ast::BinaryOp::Subtract => tacky::BinaryOp::Subtract,
        ast::BinaryOp::Multiply => tacky::BinaryOp::Multiply,
        ast::BinaryOp::Divide => tacky::BinaryOp::Divide,
        ast::BinaryOp::Remainder => tacky::BinaryOp::Remainder,
        ast::BinaryOp::BitAnd => tacky::BinaryOp::BitAnd,
        ast::BinaryOp::BitOr => tacky::BinaryOp::BitOr,
        ast::BinaryOp::BitXor => tacky::BinaryOp::BitXor,
        ast::BinaryOp::LShift => tacky::BinaryOp::LShift,
        ast::BinaryOp::RShift => tacky::BinaryOp::RShift,
    }
}
