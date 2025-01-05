use chumsky::prelude::*;
use chumsky::{error::Simple, Parser};

use crate::ast::{self, spanned};
use crate::ast::{Expr, Spanned};
use crate::lex::Token;

pub fn expr_parser<'a>(
) -> impl Parser<Token<'a>, Spanned<Expr<'a>>, Error = Simple<Token<'a>, ast::Span>> {
    let literal = select! { |span|
        Token::Number(s) => spanned(Expr::Number(s.parse().unwrap()), span),
        Token::String(s) => spanned(Expr::String(s), span),
        Token::True => spanned(Expr::Boolean(true), span),
        Token::False => spanned(Expr::Boolean(false), span),
        Token::Nil => spanned(Expr::Nil, span),
    };

    literal
}
