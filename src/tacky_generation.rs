use crate::models::ast::*;
use crate::models::symbol::*;
use crate::models::tacky;
use crate::models::tacky::Instruction;
use crate::models::tacky::TopLevel;
use crate::models::typ::Type;
use crate::utils::*;
use std::cmp::Ordering;

fn make_temp_var(typ: Type, symbols: &mut Symbols) -> tacky::Val {
    let name = make_temporary();
    symbols.insert(name, Symbol::LocalVar { typ });
    tacky::Val::Var(name)
}

pub fn convert_program(program: Program, symbols: &mut Symbols) -> tacky::Program {
    match program {
        Program::Program(decls) => {
            let decls = decls
                .into_iter()
                .filter_map(|decl| convert_decl(decl, symbols))
                .collect::<Vec<_>>();

            let top_levels = convert_symbols(symbols).into_iter().chain(decls).collect();

            tacky::Program::Program(top_levels)
        }
    }
}

fn convert_symbols(symbols: &Symbols) -> Vec<TopLevel> {
    symbols
        .iter()
        .filter_map(|(name, symbol)| {
            if let Symbol::StaticVar { typ, init, global } = symbol {
                match init {
                    InitialValue::Initial(i) => Some(TopLevel::StaticVar(name, *global, i.clone())),
                    InitialValue::Tentative => Some(TopLevel::StaticVar(
                        name,
                        *global,
                        vec![StaticInit::ZeroInit(typ.size().unwrap())],
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
        FuncDecl::FuncDecl(name, params, Some(body), Type::Func(_, ret_type), _) => {
            let mut instructions = Vec::new();

            emit_block(body, &mut instructions, symbols);

            match *ret_type {
                Type::Int => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::Int(0))))
                }
                Type::Long => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::Long(0))))
                }
                Type::UInt => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::UInt(0))))
                }
                Type::ULong | Type::Pointer(_) => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::ULong(0))))
                }
                Type::Double => {
                    instructions.push(Instruction::Return(tacky::Val::Const(Const::Double(0.))))
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
    match var_decl {
        VarDecl::VarDecl(name, Some(Initializer::Single(exp)), _, StorageClass::None) => {
            let result = emit_exp_and_convert(exp, instructions, symbols);
            instructions.push(Instruction::Copy(result, tacky::Val::Var(name)));
        }
        VarDecl::VarDecl(name, Some(init @ Initializer::Compound(_, _)), _, StorageClass::None) => {
            emit_compound_init(init, name, 0, instructions, symbols);
        }
        _ => {}
    }
}

fn emit_compound_init(
    init: Initializer,
    name: Ident,
    offset: usize,
    instructions: &mut Vec<Instruction>,
    symbols: &mut Symbols,
) {
    match init {
        Initializer::Single(e) => {
            let v = emit_exp_and_convert(e, instructions, symbols);
            instructions.push(Instruction::CopyToOffset(v, name, offset));
        }
        Initializer::Compound(Some(Type::Array(element_type, _)), inits) => {
            for (i, init) in inits.into_iter().enumerate() {
                let new_offset = offset + i * element_type.size().unwrap();
                emit_compound_init(init, name, new_offset, instructions, symbols);
            }
        }
        _ => unreachable!("Compound init has non-array type"),
    }
}

fn emit_stmt(stmt: Stmt, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) {
    match stmt {
        Stmt::Return(exp) => {
            let tacky_exp = emit_exp_and_convert(exp, instructions, symbols);
            instructions.push(Instruction::Return(tacky_exp))
        }
        Stmt::Expression(exp) => {
            emit_exp(exp, instructions, symbols);
        }
        Stmt::If(cond, then, None) => {
            let end_label = make_label("if_end");

            let cond = emit_exp_and_convert(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, end_label));
            emit_stmt(*then, instructions, symbols);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::If(cond, then, Some(else_)) => {
            let else_label = make_label("if_else");
            let end_label = make_label("if_end");

            let cond = emit_exp_and_convert(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, else_label));
            emit_stmt(*then, instructions, symbols);
            instructions.extend([Instruction::Jump(end_label), Instruction::Label(else_label)]);
            emit_stmt(*else_, instructions, symbols);
            instructions.push(Instruction::Label(end_label))
        }
        Stmt::Labelled(Label::Ident(label), stmt) => {
            instructions.push(Instruction::Label(label));
            emit_stmt(*stmt, instructions, symbols);
        }
        Stmt::Goto(label) => {
            instructions.push(Instruction::Jump(label));
        }
        Stmt::Compound(block) => {
            emit_block(block, instructions, symbols);
        }
        Stmt::Break(Some(label)) => {
            instructions.push(Instruction::Jump(Box::new(format!("break_{label}")).leak()))
        }
        Stmt::Continue(Some(label)) => {
            instructions.push(Instruction::Jump(
                Box::new(format!("continue_{label}")).leak(),
            ));
        }
        Stmt::While(cond, body, Some(label)) => {
            let break_label = Box::new(format!("break_{label}")).leak();
            let continue_label = Box::new(format!("continue_{label}")).leak();

            instructions.push(Instruction::Label(continue_label));
            let cond = emit_exp_and_convert(cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, break_label));
            emit_stmt(*body, instructions, symbols);
            instructions.extend([
                Instruction::Jump(continue_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::DoWhile(body, cond, Some(label)) => {
            let start_label = Box::new(format!("start_{label}")).leak();
            let break_label = Box::new(format!("break_{label}")).leak();
            let continue_label = Box::new(format!("continue_{label}")).leak();

            instructions.push(Instruction::Label(start_label));
            emit_stmt(*body, instructions, symbols);
            instructions.push(Instruction::Label(continue_label));
            let cond = emit_exp_and_convert(cond, instructions, symbols);
            instructions.extend([
                Instruction::JumpIfNotZero(cond, start_label),
                Instruction::Label(break_label),
            ]);
        }
        Stmt::For(init, cond, post, body, Some(label)) => {
            let start_label = Box::new(format!("start_{label}")).leak();
            let break_label = Box::new(format!("break_{label}")).leak();
            let continue_label = Box::new(format!("continue_{label}")).leak();

            match init {
                ForInit::InitDecl(decl) => emit_var_decl(decl, instructions, symbols),
                ForInit::InitExp(exp) => {
                    emit_exp(exp, instructions, symbols);
                }
                ForInit::Null => {}
            }
            instructions.push(Instruction::Label(start_label));
            if let Some(cond) = cond {
                let cond = emit_exp_and_convert(cond, instructions, symbols);
                instructions.push(Instruction::JumpIfZero(cond, break_label));
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
            let break_label = Box::new(format!("break_{label}")).leak();
            let flag = make_temp_var(cond.typ().clone(), symbols);

            let cond = emit_exp_and_convert(cond, instructions, symbols);

            for (case, case_label) in cases {
                instructions.extend([
                    Instruction::Binary(
                        tacky::BinaryOp::Equal,
                        cond,
                        tacky::Val::Const(case),
                        flag,
                    ),
                    Instruction::JumpIfNotZero(flag, case_label),
                ])
            }
            match default {
                Some(default_label) => instructions.push(Instruction::Jump(default_label)),
                None => instructions.push(Instruction::Jump(break_label)),
            }
            emit_stmt(*body, instructions, symbols);
            instructions.push(Instruction::Label(break_label));
        }
        Stmt::Null => {}
        _ => unreachable!(),
    }
}

enum ExpResult {
    PlainOperand(tacky::Val),
    DereferencedPointer(tacky::Val, Type),
}

fn emit_exp_and_convert(
    exp: Exp,
    instructions: &mut Vec<Instruction>,
    symbols: &mut Symbols,
) -> tacky::Val {
    let result = emit_exp(exp, instructions, symbols);
    match result {
        ExpResult::PlainOperand(val) => val,
        ExpResult::DereferencedPointer(ptr, typ) => {
            let dst = make_temp_var(typ, symbols);
            instructions.push(tacky::Instruction::Load(ptr, dst));
            dst
        }
    }
}

fn emit_exp(exp: Exp, instructions: &mut Vec<Instruction>, symbols: &mut Symbols) -> ExpResult {
    let Exp {
        kind,
        typ: Some(typ),
    } = exp
    else {
        unreachable!()
    };

    let op = match kind {
        ExpKind::Const(c) => tacky::Val::Const(c),
        ExpKind::Unary(op, inner) => {
            let src = emit_exp_and_convert(*inner, instructions, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.push(Instruction::Unary(op, src, dst));
            dst
        }
        ExpKind::Binary(BinaryOp::LogicalAnd, v1, v2) => {
            let false_label = make_label("and_false");
            let end_label = make_label("and_end");

            let v1 = emit_exp_and_convert(*v1, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(v1, false_label));
            let v2 = emit_exp_and_convert(*v2, instructions, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.extend([
                Instruction::JumpIfZero(v2, false_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(1)), dst),
                Instruction::Jump(end_label),
                Instruction::Label(false_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(0)), dst),
                Instruction::Label(end_label),
            ]);
            dst
        }
        ExpKind::Binary(BinaryOp::LogicalOr, v1, v2) => {
            let true_label = make_label("or_true");
            let end_label = make_label("or_end");

            let v1 = emit_exp_and_convert(*v1, instructions, symbols);
            instructions.push(Instruction::JumpIfNotZero(v1, true_label));
            let v2 = emit_exp_and_convert(*v2, instructions, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.extend([
                Instruction::JumpIfNotZero(v2, true_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(0)), dst),
                Instruction::Jump(end_label),
                Instruction::Label(true_label),
                Instruction::Copy(tacky::Val::Const(Const::Int(1)), dst),
                Instruction::Label(end_label),
            ]);
            dst
        }
        ExpKind::Binary(BinaryOp::Add, e1, e2) if typ.is_pointer() => {
            let e1_type = e1.typ().clone();

            let v1 = emit_exp_and_convert(*e1, instructions, symbols);
            let v2 = emit_exp_and_convert(*e2, instructions, symbols);
            let (ptr, index) = if typ == e1_type { (v1, v2) } else { (v2, v1) };
            let scale = typ.pointer_scale().unwrap();
            let dst = make_temp_var(typ, symbols);
            instructions.push(Instruction::AddPtr(ptr, index, scale, dst));
            dst
        }
        ExpKind::Binary(BinaryOp::Subtract, e1, e2) if typ.is_pointer() => {
            let scale = typ.pointer_scale().unwrap();

            let ptr = emit_exp_and_convert(*e1, instructions, symbols);
            let index = emit_exp_and_convert(*e2, instructions, symbols);
            let negated_index = make_temp_var(Type::Long, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.extend([
                Instruction::Unary(UnaryOp::Negate, index, negated_index),
                Instruction::AddPtr(ptr, negated_index, scale, dst),
            ]);
            dst
        }
        ExpKind::Binary(BinaryOp::Subtract, e1, e2)
            if e1.typ().is_pointer() && e2.typ().is_pointer() =>
        {
            let scale = tacky::Val::Const(Const::Long(e1.typ().pointer_scale().unwrap() as i64));
            let v1 = emit_exp_and_convert(*e1, instructions, symbols);
            let v2 = emit_exp_and_convert(*e2, instructions, symbols);
            let ptr_diff = make_temp_var(Type::Long, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.extend([
                Instruction::Binary(BinaryOp::Subtract, v1, v2, ptr_diff),
                Instruction::Binary(BinaryOp::Divide, ptr_diff, scale, dst),
            ]);
            dst
        }
        ExpKind::Binary(op, e1, e2) => {
            let v1 = emit_exp_and_convert(*e1, instructions, symbols);
            let v2 = emit_exp_and_convert(*e2, instructions, symbols);
            let dst = make_temp_var(typ, symbols);
            instructions.push(Instruction::Binary(op, v1, v2, dst));
            dst
        }
        ExpKind::Var(name) => tacky::Val::Var(name),
        ExpKind::Assignment(left, right) => {
            let lval = emit_exp(*left, instructions, symbols);
            let rval = emit_exp_and_convert(*right, instructions, symbols);

            match lval {
                ExpResult::PlainOperand(op) => {
                    instructions.push(Instruction::Copy(rval, op));
                    op
                }
                ExpResult::DereferencedPointer(ptr, _) => {
                    instructions.push(Instruction::Store(rval, ptr));
                    rval
                }
            }
        }
        ExpKind::Sequence(mut expressions) => {
            let result = expressions.pop().expect("empty sequence");
            for expression in expressions {
                emit_exp(expression, instructions, symbols);
            }
            emit_exp_and_convert(result, instructions, symbols)
        }
        ExpKind::Update { .. } => unreachable!("updates are expanded during type checking"),
        ExpKind::Conditional(cond, then, else_) => {
            let else_label = make_label("ternary_else");
            let end_label = make_label("ternary_end");
            let result = make_temp_var(typ, symbols);

            let cond = emit_exp_and_convert(*cond, instructions, symbols);
            instructions.push(Instruction::JumpIfZero(cond, else_label));
            let then = emit_exp_and_convert(*then, instructions, symbols);
            instructions.extend([
                Instruction::Copy(then, result),
                Instruction::Jump(end_label),
                Instruction::Label(else_label),
            ]);
            let else_ = emit_exp_and_convert(*else_, instructions, symbols);
            instructions.extend([
                Instruction::Copy(else_, result),
                Instruction::Label(end_label),
            ]);
            result
        }
        ExpKind::FuncCall(name, args) => {
            let result = make_temp_var(typ, symbols);

            let args = args
                .into_iter()
                .map(|arg| emit_exp_and_convert(arg, instructions, symbols))
                .collect();
            instructions.push(Instruction::FuncCall(name, args, result));
            result
        }
        ExpKind::Cast(typ, inner) => {
            let inner_type = inner.typ().clone();
            let result = emit_exp_and_convert(*inner, instructions, symbols);
            emit_cast(result, &inner_type, &typ, instructions, symbols)
        }
        ExpKind::Dereference(inner) => {
            let result = emit_exp_and_convert(*inner, instructions, symbols);
            return ExpResult::DereferencedPointer(result, typ);
        }
        ExpKind::AddressOf(inner) => {
            let result = emit_exp(*inner, instructions, symbols);
            match result {
                ExpResult::PlainOperand(op) => {
                    let dst = make_temp_var(typ, symbols);
                    instructions.push(Instruction::GetAddress(op, dst));
                    dst
                }
                ExpResult::DereferencedPointer(ptr, _) => ptr,
            }
        }
        ExpKind::Subscript(e1, e2) => {
            let pointer_first = e1.typ().is_pointer();
            let v1 = emit_exp_and_convert(*e1, instructions, symbols);
            let v2 = emit_exp_and_convert(*e2, instructions, symbols);
            let (ptr, index) = if pointer_first { (v1, v2) } else { (v2, v1) };
            let dst = make_temp_var(Type::Pointer(Box::new(typ.clone())), symbols);
            let scale = typ.size().unwrap();
            instructions.push(Instruction::AddPtr(ptr, index, scale, dst));
            return ExpResult::DereferencedPointer(dst, typ);
        }
    };

    ExpResult::PlainOperand(op)
}

fn emit_cast(
    src: tacky::Val,
    from: &Type,
    to: &Type,
    instructions: &mut Vec<Instruction>,
    symbols: &mut Symbols,
) -> tacky::Val {
    if to == from {
        return src;
    }

    let dst = make_temp_var(to.clone(), symbols);

    if from == &Type::Double {
        if to.is_signed() {
            instructions.push(Instruction::DoubleToInt(src, dst));
        } else {
            instructions.push(Instruction::DoubleToUInt(src, dst));
        }
    } else if to == &Type::Double {
        if from.is_signed() {
            instructions.push(Instruction::IntToDouble(src, dst));
        } else {
            instructions.push(Instruction::UIntToDouble(src, dst));
        }
    } else {
        match to.size().cmp(&from.size()) {
            Ordering::Equal => instructions.push(Instruction::Copy(src, dst)),
            Ordering::Less => instructions.push(Instruction::Truncate(src, dst)),
            Ordering::Greater if from.is_signed() => {
                instructions.push(Instruction::SignExtend(src, dst))
            }
            Ordering::Greater => instructions.push(Instruction::ZeroExtend(src, dst)),
        }
    }

    dst
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::{Parser, input::Input};

    fn lower(src: &str) -> Vec<TopLevel> {
        let tokens = crate::lexer::lexer().parse(src).into_result().unwrap();
        let ast = crate::parser::parser()
            .parse(
                tokens
                    .as_slice()
                    .map((src.len()..src.len()).into(), |(token, span)| (token, span)),
            )
            .into_result()
            .unwrap();
        let (ast, mut symbols) = crate::semantic_analysis::validate(ast).unwrap();
        let tacky::Program::Program(top_levels) = convert_program(ast, &mut symbols);
        top_levels
    }

    #[test]
    fn static_local_array_uses_static_initialization() {
        let top_levels = lower("int main(void) { static int a[2] = {1, 2}; return 0; }");
        assert!(top_levels.iter().any(|top_level| matches!(
            top_level,
            TopLevel::StaticVar(_, false, init)
                if matches!(init.as_slice(), [StaticInit::Int(1), StaticInit::Int(2)])
        )));
        for top_level in top_levels {
            if let TopLevel::Function(_, _, _, instructions) = top_level {
                assert!(
                    !instructions
                        .iter()
                        .any(|instruction| matches!(instruction, Instruction::CopyToOffset(..)))
                );
            }
        }
    }

    #[test]
    fn nested_local_array_uses_byte_offsets_and_zero_padding() {
        let top_levels = lower("int main(void) { long a[3][2] = {{1L}, {2L, 3L}}; return 0; }");
        let instructions = top_levels
            .iter()
            .find_map(|top_level| match top_level {
                TopLevel::Function(_, _, _, instructions) => Some(instructions),
                _ => None,
            })
            .unwrap();
        let stores = instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::CopyToOffset(tacky::Val::Const(Const::Long(value)), _, offset) => {
                    Some((*value, *offset))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(stores, [(1, 0), (0, 8), (2, 16), (3, 24), (0, 32), (0, 40)]);
    }

    #[test]
    fn subscripts_use_element_sizes_and_accept_reversed_operands() {
        let top_levels = lower("int read(int (*p)[3]) { return 1L[p][2L]; }");
        let TopLevel::Function(_, _, params, instructions) = &top_levels[0] else {
            panic!("expected function");
        };
        let additions = instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::AddPtr(ptr, _, scale, _) => Some((ptr, *scale)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            additions
                .iter()
                .map(|(_, scale)| *scale)
                .collect::<Vec<_>>(),
            [12, 4]
        );
        assert!(matches!(additions[0].0, tacky::Val::Var(name) if *name == params[0]));
    }

    #[test]
    fn pointer_difference_uses_the_pointed_to_array_size() {
        let top_levels = lower("long diff(double (*p)[3][5], double (*q)[3][5]) { return p - q; }");
        let TopLevel::Function(_, _, _, instructions) = &top_levels[0] else {
            panic!("expected function");
        };
        assert!(instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::Binary(BinaryOp::Divide, _, tacky::Val::Const(Const::Long(120)), _)
        )));
    }

    #[test]
    fn tentative_definitions_zero_initialize_the_full_object() {
        let top_levels = lower("int a[2][3]; double d; int *p;");
        for (name, size) in [("a", 24), ("d", 8), ("p", 8)] {
            assert!(top_levels.iter().any(|top_level| matches!(
                top_level,
                TopLevel::StaticVar(var, true, init)
                    if *var == name && matches!(init.as_slice(), [StaticInit::ZeroInit(bytes)] if *bytes == size)
            )));
        }
    }
}
