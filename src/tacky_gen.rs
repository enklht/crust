use crate::models::ast::*;
use crate::models::tacky;
use crate::models::tacky::Instruction;
use crate::utils::make_label;
use crate::utils::make_temporary;

pub fn convert_program(program: Program) -> tacky::Program {
    match program {
        Program::FuncDef(function) => tacky::Program::Program(convert_function(function)),
    }
}

fn convert_function(function: FuncDef) -> tacky::Function {
    match function {
        FuncDef::Function(name, body) => {
            let mut instructions = Vec::new();
            body.into_iter()
                .for_each(|item| emit_block_item(item, &mut instructions));
            instructions.push(Instruction::Return(tacky::Val::Const(0)));
            tacky::Function::Function(name, instructions)
        }
    }
}

fn emit_block_item(block_item: BlockItem, instructions: &mut Vec<Instruction>) {
    match block_item {
        BlockItem::S(stmt) => emit_stmt(stmt, instructions),
        BlockItem::D(decl) => emit_decl(decl, instructions),
    }
}

fn emit_decl(decl: Decl, instructions: &mut Vec<Instruction>) {
    match decl {
        Decl::Decl(name, Some(expr)) => {
            let result = emit_expr(expr, instructions);
            instructions.push(Instruction::Copy(result, tacky::Val::Var(name)));
        }
        Decl::Decl(_, None) => {}
    }
}

fn emit_stmt(stmt: Stmt, instructions: &mut Vec<Instruction>) {
    match stmt {
        Stmt::Return(expr) => {
            let tacky_expr = emit_expr(expr, instructions);
            instructions.push(Instruction::Return(tacky_expr))
        }
        Stmt::Expression(expr) => {
            emit_expr(expr, instructions);
        }
        Stmt::If(cond, then, None) => {
            let end_label = make_label("if_end");
            let cond = emit_expr(cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, end_label.clone()));
            emit_stmt(*then, instructions);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::If(cond, then, Some(else_)) => {
            let else_label = make_label("if_else");
            let end_label = make_label("if_end");
            let cond = emit_expr(cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            emit_stmt(*then, instructions);
            instructions.extend([
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label.clone()),
            ]);
            emit_stmt(*else_, instructions);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::Labelled(label, stmt) => {
            instructions.push(Instruction::Label(label));
            emit_stmt(*stmt, instructions);
        }
        Stmt::Goto(label) => instructions.push(Instruction::Jump(label)),
        Stmt::Null => {}
    }
}

fn emit_expr(expr: Expr, instructions: &mut Vec<Instruction>) -> tacky::Val {
    match expr {
        Expr::Const(i) => tacky::Val::Const(i),
        Expr::Unary(UnaryOp::PostfixIncrement, expr) => match *expr.clone() {
            Expr::Var(v) => {
                let tmp = tacky::Val::Var(make_temporary());
                instructions.extend([
                    Instruction::Copy(tacky::Val::Var(v.clone()), tmp.clone()),
                    Instruction::Binary(
                        tacky::BinaryOp::Add,
                        tacky::Val::Var(v.clone()),
                        tacky::Val::Const(1),
                        tacky::Val::Var(v),
                    ),
                ]);
                tmp
            }
            _ => unreachable!(),
        },
        Expr::Unary(UnaryOp::PostfixDecrement, expr) => match *expr.clone() {
            Expr::Var(v) => {
                let tmp = tacky::Val::Var(make_temporary());
                instructions.extend([
                    Instruction::Copy(tacky::Val::Var(v.clone()), tmp.clone()),
                    Instruction::Binary(
                        tacky::BinaryOp::Subtract,
                        tacky::Val::Var(v.clone()),
                        tacky::Val::Const(1),
                        tacky::Val::Var(v),
                    ),
                ]);
                tmp
            }
            _ => unreachable!(),
        },
        Expr::Unary(op, inner) => {
            let src = emit_expr(*inner, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_unop(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        Expr::Binary(BinaryOp::And, v1, v2) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");
            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_expr(*v1, instructions);
            instructions.push(Instruction::JumpIfZero(v1, false_label.clone()));
            let v2 = emit_expr(*v2, instructions);
            instructions.extend([
                Instruction::JumpIfZero(v2, false_label.clone()),
                Instruction::Copy(tacky::Val::Const(1), dst.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(false_label),
                Instruction::Copy(tacky::Val::Const(0), dst.clone()),
                Instruction::Label(end_label),
            ]);
            dst
        }
        Expr::Binary(BinaryOp::Or, v1, v2) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");
            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_expr(*v1, instructions);
            instructions.push(Instruction::JumpIfNotZero(v1, true_label.clone()));
            let v2 = emit_expr(*v2, instructions);
            instructions.extend([
                Instruction::JumpIfNotZero(v2, true_label.clone()),
                Instruction::Copy(tacky::Val::Const(0), dst.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(true_label),
                Instruction::Copy(tacky::Val::Const(1), dst.clone()),
                Instruction::Label(end_label),
            ]);
            dst
        }
        Expr::Binary(op, e1, e2) => {
            let v1 = emit_expr(*e1, instructions);
            let v2 = emit_expr(*e2, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_binop(op);
            instructions.push(Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            dst
        }
        Expr::Var(name) => tacky::Val::Var(name),
        Expr::Assignment(left, right) => match *left {
            Expr::Var(v) => {
                let result = emit_expr(*right, instructions);
                instructions.push(Instruction::Copy(result, tacky::Val::Var(v.clone())));
                tacky::Val::Var(v)
            }
            _ => unreachable!(),
        },
        Expr::Conditional(cond, then, else_) => {
            let else_label = make_label("ternary_else");
            let end_label = make_label("ternary_end");
            let result = tacky::Val::Var(make_temporary());
            let cond = emit_expr(*cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            let then = emit_expr(*then, instructions);
            instructions.extend([
                Instruction::Copy(then, result.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label),
            ]);
            let else_ = emit_expr(*else_, instructions);
            instructions.extend([
                Instruction::Copy(else_, result.clone()),
                Instruction::Label(end_label),
            ]);
            result
        }
    }
}

fn convert_unop(op: UnaryOp) -> tacky::UnaryOp {
    match op {
        UnaryOp::BitNot => tacky::UnaryOp::Complement,
        UnaryOp::Negate => tacky::UnaryOp::Negate,
        UnaryOp::Not => tacky::UnaryOp::Not,
        _ => unreachable!(),
    }
}

fn convert_binop(op: BinaryOp) -> tacky::BinaryOp {
    match op {
        BinaryOp::Add => tacky::BinaryOp::Add,
        BinaryOp::Subtract => tacky::BinaryOp::Subtract,
        BinaryOp::Multiply => tacky::BinaryOp::Multiply,
        BinaryOp::Divide => tacky::BinaryOp::Divide,
        BinaryOp::Remainder => tacky::BinaryOp::Remainder,
        BinaryOp::BitAnd => tacky::BinaryOp::BitAnd,
        BinaryOp::BitOr => tacky::BinaryOp::BitOr,
        BinaryOp::BitXor => tacky::BinaryOp::BitXor,
        BinaryOp::LShift => tacky::BinaryOp::LShift,
        BinaryOp::RShift => tacky::BinaryOp::RShift,
        BinaryOp::Equal => tacky::BinaryOp::Equal,
        BinaryOp::NotEqual => tacky::BinaryOp::NotEqual,
        BinaryOp::LessThan => tacky::BinaryOp::LessThan,
        BinaryOp::LessOrEqual => tacky::BinaryOp::LessOrEqual,
        BinaryOp::GreaterThan => tacky::BinaryOp::GreaterThan,
        BinaryOp::GreaterOrEqual => tacky::BinaryOp::GreaterOrEqual,
        _ => unreachable!(),
    }
}
