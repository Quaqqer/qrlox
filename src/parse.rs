use chumsky::prelude::*;
use chumsky::{error::Simple, Parser};

use crate::ast::{self, spanned};
use crate::ast::{Expr, Spanned};
use crate::lex::Token;

pub fn create_report<'a>(err: &'a Simple<Token<'_>, ast::Span>) -> ariadne::Report<'a> {
    ariadne::Report::build(ariadne::ReportKind::Error, err.span().range)
        .with_message(err.to_string())
        .with_label(
            ariadne::Label::new(err.span().range)
                .with_message(err.reason().to_string())
                .with_color(ariadne::Color::Red),
        )
        .finish()
}

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
