use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct Span {
    pub range: std::ops::Range<usize>,
}

impl Span {
    pub fn new(range: std::ops::Range<usize>) -> Self {
        Self { range }
    }

    pub fn join(&self, other: impl Borrow<Span>) -> Self {
        let other = other.borrow();

        Self {
            range: self.range.start.min(other.range.start)..self.range.end.max(other.range.end),
        }
    }
}

impl chumsky::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Span::new(range)
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.range.start
    }

    fn end(&self) -> Self::Offset {
        self.range.end
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.range.start, self.range.end)
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub v: T,
    pub s: Span,
}

pub fn spanned<T>(v: T, span: Span) -> Spanned<T> {
    Spanned { v, s: span }
}

#[derive(Debug, Clone, Copy)]
pub enum Binop {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

impl Binop {
    pub fn name(&self) -> &'static str {
        match self {
            Binop::Eq => "'=='",
            Binop::Ne => "'!='",
            Binop::Lt => "'<'",
            Binop::Le => "'<='",
            Binop::Gt => "'>'",
            Binop::Ge => "'>='",
            Binop::Add => "'+'",
            Binop::Sub => "'-'",
            Binop::Mul => "'*'",
            Binop::Div => "'/'",
        }
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
    String(&'a str),
    Boolean(bool),
    Nil,
    Not(Box<Spanned<Expr<'a>>>),
    Neg(Box<Spanned<Expr<'a>>>),
    Binary(Box<Spanned<Expr<'a>>>, Binop, Box<Spanned<Expr<'a>>>),
    Var(&'a str),
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Expr(Spanned<Expr<'a>>),
    Print(Spanned<Expr<'a>>),
    VarDecl(&'a str, Spanned<Expr<'a>>),
}
