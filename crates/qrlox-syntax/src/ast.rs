use std::{borrow::Borrow, rc::Rc};

#[derive(Debug, Clone)]
pub struct Span {
    range: std::ops::Range<usize>,
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

    pub fn range(&self) -> &std::ops::Range<usize> {
        &self.range
    }

    pub fn spanned<T>(&self, v: T) -> Spanned<T> {
        Spanned { v, s: self.clone() }
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

impl<T> Spanned<T> {
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            s: self.s.clone(),
            v: &self.v,
        }
    }

    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            v: f(self.v),
            s: self.s,
        }
    }
}

type Ident = Spanned<Rc<String>>;

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
    And,
    Or,
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
            Binop::And => "'and'",
            Binop::Or => "'or'",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
    Not(Box<Spanned<Expr>>),
    Neg(Box<Spanned<Expr>>),
    Binary(Box<Spanned<Expr>>, Binop, Box<Spanned<Expr>>),
    Assign(Ident, Box<Spanned<Expr>>),
    Var(Ident),
    Call(Box<Spanned<Expr>>, Vec<Spanned<Expr>>),
    Fun(Vec<Ident>, Vec<Spanned<Stmt>>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Spanned<Expr>),
    Print(Spanned<Expr>),
    VarDecl(Ident, Spanned<Expr>),
    Block(Vec<Spanned<Stmt>>),
    If {
        cond: Spanned<Expr>,
        then: Box<Spanned<Stmt>>,
        else_: Option<Box<Spanned<Stmt>>>,
    },
    While {
        cond: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    For {
        initializer: Option<Box<Spanned<Stmt>>>,
        condition: Option<Spanned<Expr>>,
        increment: Option<Spanned<Expr>>,
        body: Box<Spanned<Stmt>>,
    },
    Break,
    Continue,
    FunDecl(Ident, Vec<Ident>, Vec<Spanned<Stmt>>),
    Return(Spanned<Expr>),
}
