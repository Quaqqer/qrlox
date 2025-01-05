#[derive(Debug, Clone)]
pub struct Span {
    pub range: std::ops::Range<usize>,
}

impl Span {
    pub fn new(range: std::ops::Range<usize>) -> Self {
        Self { range }
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

#[derive(Debug)]
pub struct Spanned<T> {
    v: T,
    span: Span,
}

pub fn spanned<T>(v: T, span: Span) -> Spanned<T> {
    Spanned { v, span }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
    String(&'a str),
    Boolean(bool),
    Nil,
    Not(Box<Spanned<Expr<'a>>>),
    Neg(Box<Spanned<Expr<'a>>>),
    Binary(Box<Spanned<Expr<'a>>>, Binop, Box<Spanned<Expr<'a>>>),
}
