use crate::models::ast::*;
use crate::models::symbol::InitialValue;
use crate::models::symbol::StaticInit;
use crate::models::symbol::Symbol;
use crate::models::symbol::Symbols;
use crate::models::tacky;
use crate::models::tacky::Instruction;
use crate::models::tacky::TopLevel;
use crate::models::typ::Type;
use crate::utils::make_label;
use crate::utils::make_temporary;

fn make_temp_var(typ: Type, symbols: &mut Symbols) -> tacky::Val {
    let name = make_temporary();
    symbols.insert(name.clone(), Symbol::LocalVar { typ });
    tacky::Val::Var(name)
}

pub fn convert_program(program: Program, symbols: &mut Symbols) -> tacky::Program {
    match program {
        Program::Program(decls) => {
            let mut decls = decls
                .into_iter()
                .filter_map(|decl| convert_decl(decl, symbols))
                .collect::<Vec<_>>();
            decls.extend(convert_symbols(symbols));

            tacky::Program::Program(decls)
        }
    }
}

fn convert_symbols(symbols: &Symbols) -> Vec<TopLevel> {
    symbols
        .iter()
        .filter_map(|(name, symbol)| {
            if let Symbol::StaticVar { typ, init, global } = symbol {
                match init {
                    InitialValue::Initial(i) => {
                        Some(TopLevel::StaticVar(name.clone(), *global, i.clone()))
                    }
                    InitialValue::Tentative => Some(TopLevel::StaticVar(
                        name.clone(),
                        *global,
                        match typ {
                            Type::Int => StaticInit::Int(0),
                            Type::Long => StaticInit::Long(0),
                            _ => unreachable!(),
                        },
                    )),
                    InitialValue::NoInitializer => None,
                }
            } else {
                None
            }
        })
        .collect()
}

fn convert_decl(decl: Decl, symbols: &mut Symbols) -> Option<TopLevel> {
    match decl {
        Decl::FuncDecl(func_decl) => convert_func_decl(func_decl, symbols),
        Decl::VarDecl(_) => None,
    }
}

fn convert_func_decl(func_decl: FuncDecl, symbols: &mut Symbols) -> Option<TopLevel> {
    match func_decl {
        FuncDecl::FuncDecl(name, params, Some(body), typ, _) => {
            let mut instructions = Vec::new();
            emit_block(body, &mut instructions, symbols);
            let Type::Func(_, ret_typ) = typ else {
                unreachable!()
            };
            match *ret_typ {
                Type::Int => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::Int(0))))
                }
                Type::Long => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::Long(0))))
                }
                _ => unreachable!(),
            }

            let Symbol::Func { global, .. } = symbols.get(&name).unwrap() else {
                unreachable!()
            };
            Some(TopLevel::Function(name, *global, params, instructions))
        }
        _ => None,
    }
}

fn emit_block(block: Block, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) {
    block
        .into_iter()
        .for_each(|item| emit_block_item(item, instructions, symbols));
}

fn emit_block_item(
    block_item: BlockItem,
    instructions: &mut Vec<Instruction>,
    symbols: &mut Symbols,
) {
    match block_item {
        BlockItem::S(stmt) => emit_stmt(stmt, instructions, symbols),
        BlockItem::D(Decl::VarDecl(var_decl)) => emit_var_decl(var_decl, instructions, symbols),
        _ => {}
    }
}

fn emit_var_decl(var_decl: VarDecl, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) {
    if let VarDecl::VarDecl(name, Some(exp), _, StorageClass::None) = var_decl {
        let result = emit_exp(exp, instructions, symbols);
        instructions.push(Instruction::Copy(result, tacky::Val::Var(name)));
    }
}

fn emit_stmt(stmt: Stmt, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) {
    match stmt {
        Stmt::Return(exp) => {
            let tacky_exp = emit_exp(exp, instructions, symbols);
            instructions.push(Instruction::Return(tacky_exp))
        }
        Stmt::Expression(exp) => {
            emit_exp(exp, instructions, symbols);
        }
        Stmt::If(cond, then, None) => {
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, end_label.clone()));
            emit_stmt(*then, instructions, symbols);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::If(cond, then, Some(else_)) => {
            let else_label = make_label("if_else");
            let end_label = make_label("if_end");

            let cond = emit_exp(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            emit_stmt(*then, instructions, symbols);
            instructions.extend([
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label.clone()),
            ]);
            emit_stmt(*else_, instructions, symbols);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::Labelled(Label::Ident(label), stmt) => {
            instructions.push(Instruction::Label(label));
            emit_stmt(*stmt, instructions, symbols);
        }
        Stmt::Goto(label) => instructions.push(Instruction::Jump(label)),
        Stmt::Compound(block) => emit_block(block, instructions, symbols),
        Stmt::Break(Some(label)) => instructions.push(Instruction::Jump(format!("break_{label}"))),
        Stmt::Continue(Some(label)) => {
            instructions.push(Instruction::Jump(format!("continue_{label}")))
        }
        Stmt::While(cond, body, Some(label)) => {
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Instruction::Label(continue_label.clone()));
            let cond = emit_exp(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, break_label.clone()));
            emit_stmt(*body, instructions, symbols);
            instructions.extend([
                Instruction::Jump(continue_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::DoWhile(body, cond, Some(label)) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            instructions.push(Instruction::Label(start_label.clone()));
            emit_stmt(*body, instructions, symbols);
            instructions.push(Instruction::Label(continue_label));
            let cond = emit_exp(cond, instructions, symbols);
            instructions.extend([
                Instruction::JumpIfNotZero(cond, start_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::For(init, cond, post, body, Some(label)) => {
            let start_label = format!("start_{label}");
            let break_label = format!("break_{label}");
            let continue_label = format!("continue_{label}");

            match init {
                ForInit::InitDecl(decl) => emit_var_decl(decl, instructions, symbols),
                ForInit::InitExp(exp) => {
                    emit_exp(exp, instructions, symbols);
                }
                ForInit::Null => {}
            }
            instructions.push(Instruction::Label(start_label.clone()));
            if let Some(cond) = cond {
                let cond = emit_exp(cond, instructions, symbols);
                instructions.push(Instruction::JumpIfZero(cond, break_label.clone()));
            }
            emit_stmt(*body, instructions, symbols);
            instructions.push(Instruction::Label(continue_label));
            if let Some(post) = post {
                emit_exp(post, instructions, symbols);
            }
            instructions.extend([
                Instruction::Jump(start_label),
                Instruction::Label(break_label),
            ])
        }
        Stmt::Switch(cond, body, Some(label), Some((cases, default))) => {
            let break_label = format!("break_{label}");
            let flag = make_temp_var(cond.typ().clone(), symbols);

            let cond = emit_exp(cond, instructions, symbols);

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
            emit_stmt(*body, instructions, symbols);
            instructions.push(Instruction::Label(break_label));
        }
        Stmt::Null => {}
        _ => panic!(),
    }
}

fn emit_exp(exp: Exp, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) -> tacky::Val {
    match exp {
        Exp::Const(i) => tacky::Val::Const(i),
        Exp::Unary(op, inner, typ) => {
            let src = emit_exp(*inner, instructions, symbols);
            let dst = make_temp_var(typ.unwrap(), symbols);
            let tacky_op = convert_unop(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        Exp::Binary(BinaryOp::And, v1, v2, typ) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");

            let dst = make_temp_var(typ.unwrap(), symbols);
            let v1 = emit_exp(*v1, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(v1, false_label.clone()));
            let v2 = emit_exp(*v2, instructions, symbols);
            instructions.extend([
                Instruction::JumpIfZero(v2, false_label.clone()),
                Instruction::Copy(tacky::Val::Const(Const::Int(1)), dst.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(false_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(0)), dst.clone()),
                Instruction::Label(end_label),
            ]);
            dst
        }
        Exp::Binary(BinaryOp::Or, v1, v2, typ) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");

            let dst = make_temp_var(typ.unwrap(), symbols);
            let v1 = emit_exp(*v1, instructions, symbols);
            instructions.push(Instruction::JumpIfNotZero(v1, true_label.clone()));
            let v2 = emit_exp(*v2, instructions, symbols);
            instructions.extend([
                Instruction::JumpIfNotZero(v2, true_label.clone()),
                Instruction::Copy(tacky::Val::Const(Const::Int(0)), dst.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(true_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(1)), dst.clone()),
                Instruction::Label(end_label),
            ]);
            dst
        }
        Exp::Binary(op, e1, e2, typ) => {
            let v1 = emit_exp(*e1, instructions, symbols);
            let v2 = emit_exp(*e2, instructions, symbols);
            let dst = make_temp_var(typ.unwrap(), symbols);
            let tacky_op = convert_binop(op);
            instructions.push(Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            dst
        }
        Exp::Var(name, _) => tacky::Val::Var(name),
        Exp::Assignment(left, right, _) => match *left {
            Exp::Var(v, _) => {
                let result = emit_exp(*right, instructions, symbols);
                instructions.push(Instruction::Copy(result, tacky::Val::Var(v.clone())));
                tacky::Val::Var(v)
            }
            _ => unreachable!(),
        },
        Exp::Conditional(cond, then, else_, typ) => {
            let else_label = make_label("ternary_else");
            let end_label = make_label("ternary_end");
            let result = make_temp_var(typ.unwrap(), symbols);

            let cond = emit_exp(*cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, else_label.clone()));
            let then = emit_exp(*then, instructions, symbols);
            instructions.extend([
                Instruction::Copy(then, result.clone()),
                Instruction::Jump(end_label.clone()),
                Instruction::Label(else_label),
            ]);
            let else_ = emit_exp(*else_, instructions, symbols);
            instructions.extend([
                Instruction::Copy(else_, result.clone()),
                Instruction::Label(end_label),
            ]);
            result
        }
        Exp::FuncCall(name, args, typ) => {
            let result = make_temp_var(typ.unwrap(), symbols);

            let args = args
                .into_iter()
                .map(|arg| emit_exp(arg, instructions, symbols))
                .collect();
            instructions.push(Instruction::FuncCall(name, args, result.clone()));
            result
        }
        Exp::Cast(typ, inner) => {
            let inner_typ = inner.typ().clone();
            let result = emit_exp(*inner, instructions, symbols);
            if typ == inner_typ {
                return result;
            }

            let dst = make_temp_var(typ.clone(), symbols);

            match typ {
                Type::Int => instructions.push(Instruction::Truncate(result, dst.clone())),
                Type::Long => instructions.push(Instruction::SignExtend(result, dst.clone())),
                _ => unreachable!(),
            }

            dst
        }
    }
}

fn convert_unop(op: UnaryOp) -> tacky::UnaryOp {
    match op {
        UnaryOp::BitNot => tacky::UnaryOp::Complement,
        UnaryOp::Negate => tacky::UnaryOp::Negate,
        UnaryOp::Not => tacky::UnaryOp::Not,
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
