use crate::models::ast;
use crate::models::tacky;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn make_temporary() -> String {
    let i = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tmp.{}", i)
}

fn make_label(prefix: &str) -> String {
    let i = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}.{}", prefix, i)
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
        ast::Expr::Binary(ast::BinaryOp::And, v1, v2) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");
            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_expr(*v1, instructions);
            instructions.push(tacky::Instruction::JumpIfZero(v1, false_label.clone()));
            let v2 = emit_expr(*v2, instructions);
            instructions.extend([
                tacky::Instruction::JumpIfZero(v2, false_label.clone()),
                tacky::Instruction::Copy(tacky::Val::Const(1), dst.clone()),
                tacky::Instruction::Jump(end_label.clone()),
                tacky::Instruction::Label(false_label),
                tacky::Instruction::Copy(tacky::Val::Const(0), dst.clone()),
                tacky::Instruction::Label(end_label),
            ]);
            dst
        }
        ast::Expr::Binary(ast::BinaryOp::Or, v1, v2) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");
            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_expr(*v1, instructions);
            instructions.push(tacky::Instruction::JumpIfNotZero(v1, true_label.clone()));
            let v2 = emit_expr(*v2, instructions);
            instructions.extend([
                tacky::Instruction::JumpIfNotZero(v2, true_label.clone()),
                tacky::Instruction::Copy(tacky::Val::Const(0), dst.clone()),
                tacky::Instruction::Jump(end_label.clone()),
                tacky::Instruction::Label(true_label),
                tacky::Instruction::Copy(tacky::Val::Const(1), dst.clone()),
                tacky::Instruction::Label(end_label),
            ]);
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
        ast::UnaryOp::Not => tacky::UnaryOp::Not,
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
        ast::BinaryOp::Equal => tacky::BinaryOp::Equal,
        ast::BinaryOp::NotEqual => tacky::BinaryOp::NotEqual,
        ast::BinaryOp::LessThan => tacky::BinaryOp::LessThan,
        ast::BinaryOp::LessOrEqual => tacky::BinaryOp::LessOrEqual,
        ast::BinaryOp::GreaterThan => tacky::BinaryOp::GreaterThan,
        ast::BinaryOp::GreaterOrEqual => tacky::BinaryOp::GreaterOrEqual,
        _ => unreachable!(),
    }
}
