use crate::models::ast::*;
use crate::models::tacky;
use crate::models::tacky::Instruction as Inst;
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
            emit_block(body, &mut instructions);
            instructions.push(Inst::Return(tacky::Val::Const(0)));
            tacky::Function::Function(name, instructions)
        }
    }
}

fn emit_block(block: Block, instructions: &mut Vec<Inst>) {
    block
        .into_iter()
        .for_each(|item| emit_block_item(item, instructions));
}

fn emit_block_item(block_item: BlockItem, instructions: &mut Vec<Inst>) {
    match block_item {
        BlockItem::S(stmt) => emit_stmt(stmt, instructions),
        BlockItem::D(decl) => emit_decl(decl, instructions),
    }
}

fn emit_decl(decl: Decl, instructions: &mut Vec<Inst>) {
    match decl {
        Decl::Decl(name, Some(exp)) => {
            let result = emit_exp(exp, instructions);
            instructions.push(Inst::Copy(result, tacky::Val::Var(name)));
        }
        Decl::Decl(_, None) => {}
    }
}

fn emit_stmt(stmt: Stmt, instructions: &mut Vec<Inst>) {
    match stmt {
        Stmt::Return(exp) => {
            let tacky_exp = emit_exp(exp, instructions);
            instructions.push(Inst::Return(tacky_exp))
        }
        Stmt::Expression(exp) => {
            emit_exp(exp, instructions);
        }
        Stmt::If(cond, then, None) => {
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions);
            instructions.push(Inst::JumpIfZero(cond, end_label.clone()));
            emit_stmt(*then, instructions);
            instructions.push(Inst::Label(end_label))
        }
        Stmt::If(cond, then, Some(else_)) => {
            let else_label = make_label("if_else");
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions);
            instructions.push(Inst::JumpIfZero(cond, else_label.clone()));
            emit_stmt(*then, instructions);
            instructions.extend([
                Inst::Jump(end_label.clone()),
                Inst::Label(else_label.clone()),
            ]);
            emit_stmt(*else_, instructions);
            instructions.push(Inst::Label(end_label))
        }
        Stmt::Labelled(Label::Ident(label), stmt) => {
            instructions.push(Inst::Label(label));
            emit_stmt(*stmt, instructions);
        }
        Stmt::Goto(label) => instructions.push(Inst::Jump(label)),
        Stmt::Compound(block) => emit_block(block, instructions),
        Stmt::Break(Some(label)) => instructions.push(Inst::Jump(format!("break_{label}"))),
        Stmt::Continue(Some(label)) => instructions.push(Inst::Jump(format!("continue_{label}"))),
        Stmt::While(cond, body, Some(label)) => {
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Inst::Label(continue_label.clone()));
            let cond = emit_exp(cond, instructions);
            instructions.push(Inst::JumpIfZero(cond, break_label.clone()));
            emit_stmt(*body, instructions);
            instructions.extend([Inst::Jump(continue_label), Inst::Label(break_label)]);
        }
        Stmt::DoWhile(body, cond, Some(label)) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Inst::Label(start_label.clone()));
            emit_stmt(*body, instructions);
            instructions.push(Inst::Label(continue_label));
            let cond = emit_exp(cond, instructions);
            instructions.extend([
                Inst::JumpIfNotZero(cond, start_label),
                Inst::Label(break_label),
            ]);
        }
        Stmt::For(init, cond, post, body, Some(label)) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            match init {
                ForInit::InitDecl(decl) => emit_decl(decl, instructions),
                ForInit::InitExp(Some(exp)) => {
                    emit_exp(exp, instructions);
                }
                ForInit::InitExp(None) => {}
            }
            instructions.push(Inst::Label(start_label.clone()));
            if let Some(cond) = cond {
                let cond = emit_exp(cond, instructions);
                instructions.push(Inst::JumpIfZero(cond, break_label.clone()));
            }
            emit_stmt(*body, instructions);
            instructions.push(Inst::Label(continue_label));
            if let Some(post) = post {
                emit_exp(post, instructions);
            }
            instructions.extend([Inst::Jump(start_label), Inst::Label(break_label)])
        }
        Stmt::Null => {}
        _ => unreachable!(),
    }
}

fn emit_exp(exp: Exp, instructions: &mut Vec<Inst>) -> tacky::Val {
    match exp {
        Exp::Const(i) => tacky::Val::Const(i),
        Exp::Unary(UnaryOp::PostfixIncrement, exp) => match *exp.clone() {
            Exp::Var(v) => {
                let tmp = tacky::Val::Var(make_temporary());
                instructions.extend([
                    Inst::Copy(tacky::Val::Var(v.clone()), tmp.clone()),
                    Inst::Binary(
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
        Exp::Unary(UnaryOp::PostfixDecrement, exp) => match *exp.clone() {
            Exp::Var(v) => {
                let tmp = tacky::Val::Var(make_temporary());
                instructions.extend([
                    Inst::Copy(tacky::Val::Var(v.clone()), tmp.clone()),
                    Inst::Binary(
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
        Exp::Unary(op, inner) => {
            let src = emit_exp(*inner, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_unop(op);
            instructions.push(Inst::Unary(tacky_op, src, dst.clone()));
            dst
        }
        Exp::Binary(BinaryOp::And, v1, v2) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");

            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_exp(*v1, instructions);
            instructions.push(Inst::JumpIfZero(v1, false_label.clone()));
            let v2 = emit_exp(*v2, instructions);
            instructions.extend([
                Inst::JumpIfZero(v2, false_label.clone()),
                Inst::Copy(tacky::Val::Const(1), dst.clone()),
                Inst::Jump(end_label.clone()),
                Inst::Label(false_label),
                Inst::Copy(tacky::Val::Const(0), dst.clone()),
                Inst::Label(end_label),
            ]);
            dst
        }
        Exp::Binary(BinaryOp::Or, v1, v2) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");

            let dst = tacky::Val::Var(make_temporary());
            let v1 = emit_exp(*v1, instructions);
            instructions.push(Inst::JumpIfNotZero(v1, true_label.clone()));
            let v2 = emit_exp(*v2, instructions);
            instructions.extend([
                Inst::JumpIfNotZero(v2, true_label.clone()),
                Inst::Copy(tacky::Val::Const(0), dst.clone()),
                Inst::Jump(end_label.clone()),
                Inst::Label(true_label),
                Inst::Copy(tacky::Val::Const(1), dst.clone()),
                Inst::Label(end_label),
            ]);
            dst
        }
        Exp::Binary(op, e1, e2) => {
            let v1 = emit_exp(*e1, instructions);
            let v2 = emit_exp(*e2, instructions);
            let dst = tacky::Val::Var(make_temporary());
            let tacky_op = convert_binop(op);
            instructions.push(Inst::Binary(tacky_op, v1, v2, dst.clone()));
            dst
        }
        Exp::Var(name) => tacky::Val::Var(name),
        Exp::Assignment(left, right) => match *left {
            Exp::Var(v) => {
                let result = emit_exp(*right, instructions);
                instructions.push(Inst::Copy(result, tacky::Val::Var(v.clone())));
                tacky::Val::Var(v)
            }
            _ => unreachable!(),
        },
        Exp::Conditional(cond, then, else_) => {
            let else_label = make_label("ternary_else");
            let end_label = make_label("ternary_end");
            let result = tacky::Val::Var(make_temporary());

            let cond = emit_exp(*cond, instructions);
            instructions.push(Inst::JumpIfZero(cond, else_label.clone()));
            let then = emit_exp(*then, instructions);
            instructions.extend([
                Inst::Copy(then, result.clone()),
                Inst::Jump(end_label.clone()),
                Inst::Label(else_label),
            ]);
            let else_ = emit_exp(*else_, instructions);
            instructions.extend([Inst::Copy(else_, result.clone()), Inst::Label(end_label)]);
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
