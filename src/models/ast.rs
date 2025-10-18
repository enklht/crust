use rustc_hash::FxHashMap;

pub trait AstExtra {
    type Label: std::fmt::Debug + Clone;
    type SwitchInfo: std::fmt::Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Plain;
impl AstExtra for Plain {
    type Label = ();
    type SwitchInfo = ();
}

#[derive(Debug, Clone)]
pub struct Labelled;
impl AstExtra for Labelled {
    type Label = String;
    type SwitchInfo = ();
}

#[derive(Debug, Clone)]
pub struct SwitchResolved;
impl AstExtra for SwitchResolved {
    type Label = String;
    type SwitchInfo = (FxHashMap<i32, String>, Option<String>);
}

#[derive(Debug, Clone)]
pub enum Program<Extra: AstExtra> {
    Program(Vec<FuncDef<Extra>>),
}

#[derive(Debug, Clone)]
pub enum FuncDef<Extra: AstExtra> {
    FuncDef(String, Vec<String>, Option<Block<Extra>>),
}

#[derive(Debug, Clone)]
pub enum Decl {
    VarDecl(VarDecl),
    FuncDecl(FuncDecl),
}

#[derive(Debug, Clone)]
pub enum FuncDecl {
    FuncDecl(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    VarDecl(String, Option<Exp>),
}

pub type Block<Extra> = Vec<BlockItem<Extra>>;

#[derive(Debug, Clone)]
pub enum BlockItem<Extra: AstExtra> {
    S(Stmt<Extra>),
    D(Decl),
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
    Break(Extra::Label),
    Continue(Extra::Label),
    While(Exp, Box<Stmt<Extra>>, Extra::Label),
    DoWhile(Box<Stmt<Extra>>, Exp, Extra::Label),
    For(
        ForInit,
        Option<Exp>,
        Option<Exp>,
        Box<Stmt<Extra>>,
        Extra::Label,
    ),
    Switch(Exp, Box<Stmt<Extra>>, Extra::Label, Extra::SwitchInfo),
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
