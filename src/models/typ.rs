use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum Type {
    #[display("Int")]
    Int,
    #[display("long")]
    Long,
    #[display("Fn")]
    Func(Vec<Type>, Box<Type>),
}
