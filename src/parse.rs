use chumsky::prelude::*;
use chumsky::{error::Simple, Parser};

use crate::ast::{self, spanned, Binop, Stmt};
use crate::ast::{Expr, Spanned};
use crate::lex::Token;

pub enum StmtOrExpr<'a> {
    Stmt(Spanned<Stmt<'a>>),
    Expr(Spanned<Expr<'a>>),
}

type Error<'a> = Simple<Token<'a>, ast::Span>;

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

pub fn expr_parser<'a>() -> impl Parser<Token<'a>, Spanned<Expr<'a>>, Error = Error<'a>> {
    recursive(|expression| {
        let primary = select! { |span|
            Token::Number(s) => spanned(Expr::Number(s.parse().unwrap()), span),
            Token::String(s) => spanned(Expr::String(s), span),
            Token::True => spanned(Expr::Boolean(true), span),
            Token::False => spanned(Expr::Boolean(false), span),
            Token::Nil => spanned(Expr::Nil, span),
        }
        .or(just(Token::LParen)
            .ignore_then(expression)
            .then_ignore(just(Token::RParen)));

        let unary = recursive(|unary| {
            let not = just(Token::Bang)
                .ignore_then(unary.clone())
                .map_with_span(|e, s| spanned(Expr::Not(Box::new(e)), s));
            let neg = just(Token::Minus)
                .ignore_then(unary.clone())
                .map_with_span(|e, s| spanned(Expr::Neg(Box::new(e)), s));
            not.or(neg).or(primary.clone())
        });

        let factor = unary
            .clone()
            .then(
                just(Token::Star)
                    .to(Binop::Mul)
                    .or(just(Token::Slash).to(Binop::Div))
                    .then(unary.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        let term = factor
            .clone()
            .then(
                just(Token::Plus)
                    .to(Binop::Add)
                    .or(just(Token::Minus).to(Binop::Sub))
                    .then(factor.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        let comparison = term
            .clone()
            .then(
                just(Token::Gt)
                    .to(Binop::Gt)
                    .or(just(Token::Ge).to(Binop::Ge))
                    .or(just(Token::Lt).to(Binop::Lt))
                    .or(just(Token::Le).to(Binop::Le))
                    .then(term.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        let equality = comparison
            .clone()
            .then(
                just(Token::EqEq)
                    .to(Binop::Eq)
                    .or(just(Token::BangEq).to(Binop::Ne))
                    .then(term.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        equality
    })
}

pub fn stmt_parser<'a>() -> impl Parser<Token<'a>, Spanned<Stmt<'a>>, Error = Error<'a>> {
    let print = just(Token::Print)
        .ignore_then(expr_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with_span(|expr, span| spanned(Stmt::Print(expr), span));

    let expr = expr_parser()
        .then_ignore(just(Token::Semicolon))
        .map_with_span(|expr, span| spanned(Stmt::Expr(expr), span));

    print.or(expr)
}

pub fn expr_or_stmt_parser<'a>() -> impl Parser<Token<'a>, StmtOrExpr<'a>, Error = Error<'a>> {
    expr_parser()
        .map(StmtOrExpr::Expr)
        .or(stmt_parser().map(StmtOrExpr::Stmt))
}
