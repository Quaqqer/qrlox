use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\n\f]*")]
pub enum Token<'a> {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,

    #[token("!")]
    Bang,
    #[token("!=")]
    BangEq,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[regex(r#"[A-Za-z_][A-Za-z_0-9]*"#)]
    Identifier(&'a str),
    #[regex(r#""((\\")|[^"])*""#, |s| &s.slice()[1..s.slice().len()-1])]
    String(&'a str),
    #[regex(r#"(\d+(\.\d*)?|\.\d+)"#)]
    Number(&'a str),
    #[token("and")]
    And,
    #[token("class")]
    Class,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("fun")]
    Fun,
    #[token("for")]
    For,
    #[token("if")]
    If,
    #[token("nil")]
    Nil,
    #[token("or")]
    Or,
    #[token("print")]
    Print,
    #[token("return")]
    Return,
    #[token("super")]
    Super,
    #[token("this")]
    This,
    #[token("true")]
    True,
    #[token("var")]
    Var,
    #[token("while")]
    While,

    Error,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::LParen => "'('",
                Token::RParen => "')'",
                Token::LBrace => "'{'",
                Token::RBrace => "'}'",
                Token::Comma => "','",
                Token::Dot => "'.'",
                Token::Minus => "'-'",
                Token::Plus => "'+'",
                Token::Semicolon => "';'",
                Token::Slash => "'/'",
                Token::Star => "'*'",
                Token::Bang => "'!'",
                Token::BangEq => "'!='",
                Token::Eq => "'='",
                Token::EqEq => "'=='",
                Token::Gt => "'>'",
                Token::Ge => "'>='",
                Token::Lt => "'<'",
                Token::Le => "'<='",
                Token::Identifier(_) => "identifier",
                Token::String(_) => "string",
                Token::Number(_) => "number",
                Token::And => "'and'",
                Token::Class => "'class'",
                Token::Else => "'else'",
                Token::False => "'false'",
                Token::Fun => "'fun'",
                Token::For => "'for'",
                Token::If => "'if'",
                Token::Nil => "'nil'",
                Token::Or => "'or'",
                Token::Print => "'print'",
                Token::Return => "'return'",
                Token::Super => "'super'",
                Token::This => "'this'",
                Token::True => "'true'",
                Token::Var => "'var'",
                Token::While => "'while'",
                Token::Error => "'error'",
            }
        )
    }
}
