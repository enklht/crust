use crate::models::ast::*;
use crate::models::tacky;
use crate::models::tacky::Instruction;
use crate::utils::make_label;
use crate::utils::make_temporary;

fn make_temp_var() -> tacky::Val {
    tacky::Val::Var(make_temporary())
}

pub fn convert_program(program: Program<SwitchResolved>) -> tacky::Program {
    match program {
        Program::Program(func_defs) => {
            tacky::Program::Program(func_defs.into_iter().filter_map(convert_func_def).collect())
        }
    }
}

fn convert_func_def(func_def: FuncDef<SwitchResolved>) -> Option<tacky::FuncDef> {
    match func_def {
        FuncDef::FuncDef(name, params, Some(body)) => {
            let mut instructions = Vec::new();
            emit_block(body, &mut instructions);
            instructions.push(Instruction::Return(tacky::Val::Const(0)));
            Some(tacky::FuncDef::FuncDef(name, params, instructions))
        }
        _ => None,
    }
}

fn emit_block(block: Block<SwitchResolved>, instructions: &mut Vec<Instruction>) {
    block
        .into_iter()
        .for_each(|item| emit_block_item(item, instructions));
}

fn emit_block_item(block_item: BlockItem<SwitchResolved>, instructions: &mut Vec<Instruction>) {
    match block_item {
        BlockItem::S(stmt) => emit_stmt(stmt, instructions),
        BlockItem::D(Decl::VarDecl(var_decl)) => emit_var_decl(var_decl, instructions),
        _ => {}
    }
}

fn emit_var_decl(var_decl: VarDecl, instructions: &mut Vec<Instruction>) {
    if let VarDecl::VarDecl(name, Some(exp)) = var_decl {
        let result = emit_exp(exp, instructions);
        instructions.push(Instruction::Copy(result, tacky::Val::Var(name)));
    }
}

fn emit_stmt(stmt: Stmt<SwitchResolved>, instructions: &mut Vec<Instruction>) {
    match stmt {
        Stmt::Return(exp) => {
            let tacky_exp = emit_exp(exp, instructions);
            instructions.push(Instruction::Return(tacky_exp))
        }
        Stmt::Expression(exp) => {
            emit_exp(exp, instructions);
        }
        Stmt::If(cond, then, None) => {
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, end_label.clone()));
            emit_stmt(*then, instructions);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::If(cond, then, Some(else_)) => {
            let else_label = make_label("if_else");
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            emit_stmt(*then, instructions);
            instructions.extend([
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label.clone()),
            ]);
            emit_stmt(*else_, instructions);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::Labelled(Label::Ident(label), stmt) => {
            instructions.push(Instruction::Label(label));
            emit_stmt(*stmt, instructions);
        }
        Stmt::Goto(label) => instructions.push(Instruction::Jump(label)),
        Stmt::Compound(block) => emit_block(block, instructions),
        Stmt::Break(label) => instructions.push(Instruction::Jump(format!("break_{label}"))),
        Stmt::Continue(label) => instructions.push(Instruction::Jump(format!("continue_{label}"))),
        Stmt::While(cond, body, label) => {
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Instruction::Label(continue_label.clone()));
            let cond = emit_exp(cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, break_label.clone()));
            emit_stmt(*body, instructions);
            instructions.extend([
                Instruction::Jump(continue_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::DoWhile(body, cond, label) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Instruction::Label(start_label.clone()));
            emit_stmt(*body, instructions);
            instructions.push(Instruction::Label(continue_label));
            let cond = emit_exp(cond, instructions);
            instructions.extend([
                Instruction::JumpIfNotZero(cond, start_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::For(init, cond, post, body, label) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            match init {
                ForInit::InitDecl(decl) => emit_var_decl(decl, instructions),
                ForInit::InitExp(exp) => {
                    emit_exp(exp, instructions);
                }
                ForInit::Null => {}
            }
            instructions.push(Instruction::Label(start_label.clone()));
            if let Some(cond) = cond {
                let cond = emit_exp(cond, instructions);
                instructions.push(Instruction::JumpIfZero(cond, break_label.clone()));
            }
            emit_stmt(*body, instructions);
            instructions.push(Instruction::Label(continue_label));
            if let Some(post) = post {
                emit_exp(post, instructions);
            }
            instructions.extend([
                Instruction::Jump(start_label),
                Instruction::Label(break_label),
            ])
        }
        Stmt::Switch(cond, body, label, (cases, default)) => {
            let break_label = format!("break_{label}");
            let cond = emit_exp(cond, instructions);
            let flag = make_temp_var();

            for (case, case_label) in cases {
                instructions.extend([
                    Instruction::Binary(
                        tacky::BinaryOp::Equal,
                        cond.clone(),
                        tacky::Val::Const(case),
                        flag.clone(),
                    ),
                    Instruction::JumpIfNotZero(flag.clone(), case_label),
                ])
            }
            match default {
                Some(default_label) => instructions.push(Instruction::Jump(default_label)),
                None => instructions.push(Instruction::Jump(break_label.clone())),
            }
            emit_stmt(*body, instructions);
            instructions.push(Instruction::Label(break_label));
        }
        Stmt::Null => {}
        _ => unreachable!(),
    }
}

fn emit_exp(exp: Exp, instructions: &mut Vec<Instruction>) -> tacky::Val {
    match exp {
        Exp::Const(i) => tacky::Val::Const(i),
        Exp::Unary(UnaryOp::PostfixIncrement, exp) => match *exp.clone() {
            Exp::Var(v) => {
                let tmp = make_temp_var();
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
        Exp::Unary(UnaryOp::PostfixDecrement, exp) => match *exp.clone() {
            Exp::Var(v) => {
                let tmp = make_temp_var();
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
        Exp::Unary(op, inner) => {
            let src = emit_exp(*inner, instructions);
            let dst = make_temp_var();
            let tacky_op = convert_unop(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        Exp::Binary(BinaryOp::And, v1, v2) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");

            let dst = make_temp_var();
            let v1 = emit_exp(*v1, instructions);
            instructions.push(Instruction::JumpIfZero(v1, false_label.clone()));
            let v2 = emit_exp(*v2, instructions);
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
        Exp::Binary(BinaryOp::Or, v1, v2) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");

            let dst = make_temp_var();
            let v1 = emit_exp(*v1, instructions);
            instructions.push(Instruction::JumpIfNotZero(v1, true_label.clone()));
            let v2 = emit_exp(*v2, instructions);
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
        Exp::Binary(op, e1, e2) => {
            let v1 = emit_exp(*e1, instructions);
            let v2 = emit_exp(*e2, instructions);
            let dst = make_temp_var();
            let tacky_op = convert_binop(op);
            instructions.push(Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            dst
        }
        Exp::Var(name) => tacky::Val::Var(name),
        Exp::Assignment(left, right) => match *left {
            Exp::Var(v) => {
                let result = emit_exp(*right, instructions);
                instructions.push(Instruction::Copy(result, tacky::Val::Var(v.clone())));
                tacky::Val::Var(v)
            }
            _ => unreachable!(),
        },
        Exp::Conditional(cond, then, else_) => {
            let else_label = make_label("ternary_else");
            let end_label = make_label("ternary_end");
            let result = make_temp_var();

            let cond = emit_exp(*cond, instructions);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            let then = emit_exp(*then, instructions);
            instructions.extend([
                Instruction::Copy(then, result.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label),
            ]);
            let else_ = emit_exp(*else_, instructions);
            instructions.extend([
                Instruction::Copy(else_, result.clone()),
                Instruction::Label(end_label),
            ]);
            result
        }
        Exp::FuncCall(name, args) => {
            let result = make_temp_var();

            let args = args
                .into_iter()
                .map(|arg| emit_exp(arg, instructions))
                .collect();
            instructions.push(Instruction::FuncCall(name, args, result.clone()));
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
