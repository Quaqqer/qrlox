use std::rc::Rc;

use chumsky::prelude::*;
use chumsky::{error::Simple, Parser};

use crate::ast::{self, spanned, Binop, Stmt};
use crate::ast::{Expr, Spanned};
use crate::token::Token;
use crate::ProgramOrExpr;

type Error<'a> = Simple<Token<'a>, ast::Span>;

#[allow(clippy::let_and_return)]
pub fn expr_parser<'a>() -> impl Parser<Token<'a>, Spanned<Expr>, Error = Error<'a>> {
    recursive(|expression| {
        let primary = select! { |span|
            Token::Number(s) => spanned(Expr::Number(s.parse().unwrap()), span),
            Token::String(s) => spanned(Expr::String(Rc::new(s.to_string())), span),
            Token::Identifier(i) => spanned(Expr::Var(Rc::new(i.to_string())), span),
            Token::True => spanned(Expr::Boolean(true), span),
            Token::False => spanned(Expr::Boolean(false), span),
            Token::Nil => spanned(Expr::Nil, span),
        }
        .or(just(Token::LParen)
            .ignore_then(expression.clone())
            .then_ignore(just(Token::RParen)));

        let call = primary
            .clone()
            .then(
                just(Token::LParen)
                    .ignore_then(expression.clone().separated_by(just(Token::Comma)))
                    .then_ignore(just(Token::RParen))
                    .map_with_span(spanned)
                    .repeated(),
            )
            .foldl(|lhs, args| {
                let s = lhs.s.join(args.s);
                spanned(Expr::Call(Box::new(lhs), args.v), s)
            });

        let unary = recursive(|unary| {
            let not = just(Token::Bang)
                .ignore_then(unary.clone())
                .map_with_span(|e, s| spanned(Expr::Not(Box::new(e)), s));
            let neg = just(Token::Minus)
                .ignore_then(unary.clone())
                .map_with_span(|e, s| spanned(Expr::Neg(Box::new(e)), s));
            not.or(neg).or(call)
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

        let and = equality
            .clone()
            .then(
                just(Token::And)
                    .to(Binop::And)
                    .then(equality.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        let or = and
            .clone()
            .then(just(Token::Or).to(Binop::Or).then(and.clone()).repeated())
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.s.join(&rhs.s);
                spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
            });

        let assignment = select!(Token::Identifier(i) => i)
            .then_ignore(just(Token::Eq))
            .then(or.clone())
            .map_with_span(|(var, expr), span| {
                spanned(Expr::Assign(Rc::new(var.to_string()), Box::new(expr)), span)
            })
            .or(or);

        assignment
    })
}

fn var_decl<'a>() -> impl Parser<Token<'a>, Spanned<Stmt>, Error = Error<'a>> {
    just(Token::Var)
        .ignore_then(select! {Token::Identifier(ident) => ident})
        .then_ignore(just(Token::Eq))
        .then(expr_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with_span(|(var, expr), s| spanned(Stmt::VarDecl(Rc::new(var.to_string()), expr), s))
}

fn expr_stmt<'a>() -> impl Parser<Token<'a>, Spanned<Stmt>, Error = Error<'a>> {
    expr_parser()
        .then_ignore(just(Token::Semicolon))
        .map_with_span(|expr, span| spanned(Stmt::Expr(expr), span))
}

fn stmt_parser<'a>() -> impl Parser<Token<'a>, Spanned<Stmt>, Error = Error<'a>> {
    recursive(|stmt_parser| {
        let print = just(Token::Print)
            .ignore_then(expr_parser())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|expr, span| spanned(Stmt::Print(expr), span));

        let block = just(Token::LBrace)
            .ignore_then(stmt_parser.clone().repeated())
            .then_ignore(just(Token::RBrace))
            .map_with_span(|stmts, s| spanned(Stmt::Block(stmts), s));

        let if_ = just(Token::If)
            .then(just(Token::LParen))
            .ignore_then(expr_parser())
            .then_ignore(just(Token::RParen))
            .then(stmt_parser.clone())
            .then(just(Token::Else).ignore_then(stmt_parser.clone()).or_not())
            .map_with_span(|((cond, then), else_), s| {
                spanned(
                    Stmt::If {
                        cond,
                        then: Box::new(then),
                        else_: else_.map(Box::new),
                    },
                    s,
                )
            });

        let while_ = just(Token::While)
            .ignore_then(just(Token::LParen))
            .ignore_then(expr_parser())
            .then_ignore(just(Token::RParen))
            .then(stmt_parser.clone())
            .map_with_span(|(cond, body), span| {
                spanned(
                    Stmt::While {
                        cond,
                        body: Box::new(body),
                    },
                    span,
                )
            });

        let for_ = just(Token::For)
            .ignore_then(just(Token::LParen))
            .ignore_then(
                var_decl()
                    .or(expr_stmt())
                    .or_not()
                    .or(just(Token::Semicolon).to(None)),
            )
            .then(expr_parser().or_not())
            .then_ignore(just(Token::Semicolon))
            .then(expr_parser().or_not())
            .then_ignore(just(Token::RParen))
            .then(stmt_parser.clone())
            .map_with_span(|(((initializer, condition), increment), body), s| {
                spanned(
                    Stmt::For {
                        initializer: initializer.map(Box::new),
                        condition,
                        increment,
                        body: Box::new(body),
                    },
                    s,
                )
            });

        let break_ = just(Token::Break)
            .then(just(Token::Semicolon))
            .ignored()
            .map_with_span(|(), s| spanned(Stmt::Break, s));

        let continue_ = just(Token::Continue)
            .then(just(Token::Semicolon))
            .ignored()
            .map_with_span(|(), s| spanned(Stmt::Continue, s));

        let fun = just(Token::Fun)
            .ignore_then(select! {Token::Identifier(ident)=>ident})
            .then_ignore(just(Token::LParen))
            .then(
                select! {Token::Identifier(ident)=>ident}
                    .map_with_span(spanned)
                    .separated_by(just(Token::Comma)),
            )
            .then_ignore(just(Token::RParen))
            .then_ignore(just(Token::LBrace))
            .then(stmt_parser.clone().repeated())
            .then_ignore(just(Token::RBrace))
            .map_with_span(|((name, args), body), s| {
                spanned(
                    Stmt::FunDecl(
                        Rc::new(name.to_string()),
                        args.iter()
                            .map(|arg| arg.clone().map(|s| Rc::new(s.to_string())))
                            .collect::<Vec<_>>(),
                        body,
                    ),
                    s,
                )
            });

        let return_ = just(Token::Return)
            .ignore_then(expr_parser())
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|expr, s| spanned(Stmt::Return(expr), s));

        print
            .or(var_decl())
            .or(block)
            .or(if_)
            .or(while_)
            .or(for_)
            .or(break_)
            .or(continue_)
            .or(fun)
            .or(return_)
            .or(expr_stmt())
    })
}

pub(crate) fn expr_or_program_parser<'a>(
) -> impl Parser<Token<'a>, ProgramOrExpr, Error = Error<'a>> {
    expr_parser()
        .map(ProgramOrExpr::Expr)
        .then_ignore(end())
        .or(program_parser().map(ProgramOrExpr::Program))
}

pub(crate) fn program_parser<'a>() -> impl Parser<Token<'a>, Vec<Spanned<Stmt>>, Error = Error<'a>>
{
    stmt_parser().repeated().then_ignore(end())
}
