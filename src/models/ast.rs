use rustc_hash::FxHashMap;

pub trait AstExtra {
    type Switch: std::fmt::Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Plain;
impl AstExtra for Plain {
    type Switch = ();
}

#[derive(Debug, Clone)]
pub struct SwitchResolved;
impl AstExtra for SwitchResolved {
    type Switch = (FxHashMap<i32, String>, Option<String>);
}

#[derive(Debug, Clone)]
pub enum Program<Extra: AstExtra> {
    Program(Vec<FuncDecl<Extra>>),
}

#[derive(Debug, Clone)]
pub enum Decl<Extra: AstExtra> {
    VarDecl(VarDecl),
    FuncDecl(FuncDecl<Extra>),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(String, Option<Exp>),
}

#[derive(Debug, Clone)]
pub enum FuncDecl<Extra: AstExtra> {
    FuncDecl(String, Vec<String>, Option<Block<Extra>>),
}

pub type Block<Extra> = Vec<BlockItem<Extra>>;

#[derive(Debug, Clone)]
pub enum BlockItem<Extra: AstExtra> {
    S(Stmt<Extra>),
    D(Decl<Extra>),
}

#[derive(Debug, Clone)]
pub enum ForInit {
    InitDecl(VarDecl),
    InitExp(Exp),
    Null,
}

#[derive(Debug, Clone)]
pub enum Label {
    Ident(String),
    Case(Exp),
    Default,
}

#[derive(Debug, Clone)]
pub enum Stmt<Extra: AstExtra> {
    Return(Exp),
    Expression(Exp),
    If(Exp, Box<Stmt<Extra>>, Option<Box<Stmt<Extra>>>),
    Labelled(Label, Box<Stmt<Extra>>),
    Goto(String),
    Compound(Block<Extra>),
    Break(Option<String>),
    Continue(Option<String>),
    While(Exp, Box<Stmt<Extra>>, Option<String>),
    DoWhile(Box<Stmt<Extra>>, Exp, Option<String>),
    For(
        ForInit,
        Option<Exp>,
        Option<Exp>,
        Box<Stmt<Extra>>,
        Option<String>,
    ),
    Switch(Exp, Box<Stmt<Extra>>, Option<String>, Extra::Switch),
    Null,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Const(i32),
    Var(String),
    Unary(UnaryOp, Box<Exp>),
    Binary(BinaryOp, Box<Exp>, Box<Exp>),
    Assignment(Box<Exp>, Box<Exp>),
    Conditional(Box<Exp>, Box<Exp>, Box<Exp>),
    FuncCall(String, Vec<Exp>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    BitNot,
    Not,
    PostfixIncrement,
    PostfixDecrement,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}
