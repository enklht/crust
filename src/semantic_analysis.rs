mod block_labelling;
mod ident_resolution;
mod label_resolution;
mod switch_resolution;
mod type_check;

use crate::models::ast::*;
use crate::models::symbol::*;
use crate::models::typ::Types;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0}")]
    Label(String),
    #[error("{0}")]
    FuncDecl(String),
    #[error("{0}")]
    VarDecl(String),
    #[error("{0}")]
    Type(String),
    #[error("{0}")]
    LoopLabel(String),
    #[error("{0}")]
    Switch(String),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate(mut program: Program, types: &mut Types) -> ValidationResult<(Program, Symbols)> {
    ident_resolution::resolve_program(&mut program)?;

    let mut program = label_resolution::resolve_program(program)?;

    let symbols = type_check::check_program(&mut program, types)?;

    block_labelling::label_program(&mut program)?;
    switch_resolution::resolve_program(&mut program, types)?;

    Ok((program, symbols))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::typ::TypeKind;
    use chumsky::Parser;
    use chumsky::input::Input;
    use chumsky::inspector::SimpleState;

    #[test]
    fn array_parameter_adjustment_preserves_interned_types() {
        let source = "
            int read(int values[2][3]);
            int read(int (*values)[3]) { return values[0][0]; }
            int main(void) {
                int values[2][3] = {{1}};
                return read(values);
            }
        ";
        let tokens = crate::lexer::lexer().parse(source).into_result().unwrap();
        let mut state = SimpleState(Types::new());
        let program = crate::parser::parser()
            .parse_with_state(
                tokens
                    .as_slice()
                    .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
                &mut state,
            )
            .into_result()
            .unwrap();
        let mut types = state.0;
        let Program::Program(decls) = &program;
        let Decl::FuncDecl(FuncDecl::FuncDecl(_, _, _, parsed_type, _)) = &decls[0] else {
            panic!("expected function declaration")
        };
        let parsed_type = *parsed_type;
        let row = types.intern(TypeKind::Array(types.int, 3));
        let matrix = types.intern(TypeKind::Array(row, 2));
        let ptr = types.intern(TypeKind::Pointer(row));
        let adjusted_type = types.intern(TypeKind::Func(vec![ptr], types.int));

        let (_, symbols) = validate(program, &mut types).unwrap();

        assert_eq!(symbols["read"].typ(), adjusted_type);
        assert_eq!(
            types.kind(parsed_type),
            &TypeKind::Func(vec![matrix], types.int)
        );
    }
}
