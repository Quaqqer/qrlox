use std::rc::Rc;

use chumsky::input::{MapExtra, ValueInput};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::ast::{self, Binop, ClassDecl, FunDecl, Span, Stmt};
use crate::ast::{Expr, Spanned};
use crate::token::Token;
use crate::{spanned, ProgramOrExpr};

fn to_spanned<'a, 'b, I, T>(v: T, extra: &mut MapExtra<'a, 'b, I, Err<'a>>) -> Spanned<T>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    ast::spanned(v, extra.span())
}

// trait In<'a>: ValueInput<'a, Token = Token<'a>, Span = Span> {}
type Err<'a> = extra::Err<Rich<'a, Token<'a>, Span>>;

#[allow(clippy::let_and_return)]
pub fn expr_parser<'a, I>(
    stmt_parser: impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone + 'a,
    expr_parser: impl Parser<'a, I, Spanned<Expr>, Err<'a>> + Clone + 'a,
) -> impl Parser<'a, I, Spanned<Expr>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let var = ident().map(Expr::Var).map_with(to_spanned);
    let true_ = just(Token::True)
        .to(Expr::Boolean(true))
        .map_with(to_spanned);
    let false_ = just(Token::False)
        .to(Expr::Boolean(false))
        .map_with(to_spanned);
    let nil = just(Token::Nil).to(Expr::Nil).map_with(to_spanned);
    let grouping = expr_parser
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));
    let fun = just(Token::Fun)
        .ignore_then(
            ident()
                .separated_by(just(Token::Comma))
                .collect()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(
            stmt_parser
                .clone()
                .repeated()
                .collect()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|(args, body)| Expr::Fun(args, body))
        .map_with(to_spanned);

    let primary = select! {
        Token::Number(s) => Expr::Number(s.parse().unwrap()),
        Token::String(s) => Expr::String(Rc::new(s.to_string())),
    }
    .map_with(to_spanned)
    .or(var)
    .or(true_)
    .or(false_)
    .or(nil)
    .or(grouping)
    .or(fun);

    #[derive(Clone)]
    enum CallFoldable {
        Call { args: Vec<Spanned<Expr>> },
        Get { field: Spanned<Rc<String>> },
    }

    let call = primary.foldl(
        {
            let call = expr_parser
                .clone()
                .separated_by(just(Token::Comma))
                .collect()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|args| CallFoldable::Call { args })
                .labelled("function call")
                .as_context();
            let get = just(Token::Dot)
                .ignore_then(ident())
                .map(|field| CallFoldable::Get { field })
                .labelled("field access")
                .as_context();
            call.or(get).map_with(to_spanned).repeated()
        },
        |lhs, foldable| {
            let s = lhs.s.join(foldable.s);
            match foldable.v {
                CallFoldable::Call { args } => spanned(Expr::Call(Box::new(lhs), args), s),
                CallFoldable::Get { field } => spanned(Expr::Get(Box::new(lhs), field), s),
            }
        },
    );

    let unary = recursive(|unary| {
        let not = just(Token::Bang)
            .ignore_then(unary.clone())
            .map(|e| Expr::Not(Box::new(e)))
            .map_with(to_spanned);
        let neg = just(Token::Minus)
            .ignore_then(unary.clone())
            .map(|e| Expr::Neg(Box::new(e)))
            .map_with(to_spanned);
        not.or(neg).or(call.clone())
    });

    let factor = unary.clone().foldl(
        just(Token::Star)
            .to(Binop::Mul)
            .or(just(Token::Slash).to(Binop::Div))
            .then(unary.clone())
            .repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let term = factor.clone().foldl(
        just(Token::Plus)
            .to(Binop::Add)
            .or(just(Token::Minus).to(Binop::Sub))
            .then(factor.clone())
            .repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let comparison = term.clone().foldl(
        just(Token::Gt)
            .to(Binop::Gt)
            .or(just(Token::Ge).to(Binop::Ge))
            .or(just(Token::Lt).to(Binop::Lt))
            .or(just(Token::Le).to(Binop::Le))
            .then(term.clone())
            .repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let equality = comparison.clone().foldl(
        just(Token::EqEq)
            .to(Binop::Eq)
            .or(just(Token::BangEq).to(Binop::Ne))
            .then(term.clone())
            .repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let and = equality.clone().foldl(
        just(Token::And)
            .to(Binop::And)
            .then(equality.clone())
            .repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let or = and.clone().foldl(
        just(Token::Or).to(Binop::Or).then(and.clone()).repeated(),
        |lhs, (op, rhs)| {
            let s = lhs.s.join(&rhs.s);
            spanned(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), s)
        },
    );

    let var_assignment = ident()
        .then_ignore(just(Token::Eq))
        .then(or.clone())
        .map(|(ident, expr)| Expr::Assign(ident, Box::new(expr)))
        .map_with(to_spanned);
    let field_assignment = call
        .clone()
        .then_ignore(just(Token::Eq))
        .then(or.clone())
        .try_map(|(lhs, rhs), s| match lhs.v {
            Expr::Get(lhs, field) => Ok(Expr::Set(lhs, field, Box::new(rhs))),
            _ => Err(Rich::custom(s, "Cannot assign to function calls.")),
        })
        .map_with(to_spanned);
    let assignment = var_assignment.or(field_assignment).or(or);

    assignment.labelled("expression").as_context()
}

fn ident<'a, I>() -> impl Parser<'a, I, Spanned<Rc<String>>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    select! {
        Token::Identifier(ident) => ident,
    }
    .map(|name| Rc::new(name.to_string()))
    .map_with(to_spanned)
    .labelled("identifier")
    .as_context()
}

fn var_decl<'a, I>(
    expr_parser: impl Parser<'a, I, Spanned<Expr>, Err<'a>> + Clone,
) -> impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    just(Token::Var)
        .ignore_then(ident())
        .then_ignore(just(Token::Eq))
        .then(expr_parser)
        .then_ignore(just(Token::Semicolon))
        .map(|(ident, expr)| Stmt::VarDecl(ident, expr))
        .map_with(to_spanned)
        .labelled("variable declaration")
        .as_context()
}

fn fun_decl<'a, I>(
    stmt_parser: impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone + 'a,
) -> impl Parser<'a, I, Spanned<FunDecl>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    just(Token::Fun)
        .ignore_then(ident())
        .then_ignore(just(Token::LParen))
        .then(ident().separated_by(just(Token::Comma)).collect())
        .then_ignore(just(Token::RParen))
        .then_ignore(just(Token::LBrace))
        .then(stmt_parser.clone().repeated().collect())
        .then_ignore(just(Token::RBrace))
        .map(|((name, params), body)| FunDecl { name, params, body })
        .map_with(to_spanned)
        .labelled("function declaration")
        .as_context()
}

fn class_decl<'a, I>(
    stmt_parser: impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone + 'a,
) -> impl Parser<'a, I, Spanned<ClassDecl>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    just(Token::Class)
        .ignore_then(ident())
        .then_ignore(just(Token::LBrace))
        .then(fun_decl(stmt_parser).repeated().collect())
        .then_ignore(just(Token::RBrace))
        .map(|(ident, functions)| ClassDecl { ident, functions })
        .map_with(to_spanned)
        .labelled("class declaration")
        .as_context()
}

fn expr_stmt<'a, I>(
    expr_parser: impl Parser<'a, I, Spanned<Expr>, Err<'a>> + Clone,
) -> impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    expr_parser
        .then_ignore(just(Token::Semicolon))
        .map(|expr| Stmt::Expr(expr))
        .map_with(to_spanned)
}

fn stmt_parser<'a, I>(
    stmt_parser: impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone + 'a,
    expr_parser: impl Parser<'a, I, Spanned<Expr>, Err<'a>> + Clone + 'a,
) -> impl Parser<'a, I, Spanned<Stmt>, Err<'a>> + Clone
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let print = just(Token::Print)
        .ignore_then(expr_parser.clone())
        .then_ignore(just(Token::Semicolon))
        .map(Stmt::Print)
        .map_with(to_spanned)
        .labelled("print statement")
        .as_context();

    let block = stmt_parser
        .clone()
        .repeated()
        .collect()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map(Stmt::Block)
        .map_with(to_spanned)
        .labelled("block statement")
        .as_context();

    let if_ = just(Token::If)
        .ignore_then(
            expr_parser
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt_parser.clone())
        .then(just(Token::Else).ignore_then(stmt_parser.clone()).or_not())
        .map(|((cond, then), else_)| Stmt::If {
            cond,
            then: Box::new(then),
            else_: else_.map(Box::new),
        })
        .map_with(to_spanned)
        .labelled("if statement")
        .as_context();

    let while_ = just(Token::While)
        .ignore_then(
            expr_parser
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(stmt_parser.clone())
        .map(|(cond, body)| Stmt::While {
            cond,
            body: Box::new(body),
        })
        .map_with(to_spanned)
        .labelled("while loop")
        .as_context();

    let for_ = just(Token::For)
        .ignore_then(just(Token::LParen))
        .ignore_then(
            var_decl(expr_parser.clone())
                .or(expr_stmt(expr_parser.clone()))
                .or_not()
                .or(just(Token::Semicolon).to(None))
                .then(expr_parser.clone().or_not())
                .then_ignore(just(Token::Semicolon))
                .then(expr_parser.clone().or_not())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then_ignore(just(Token::RParen))
        .then(stmt_parser.clone())
        .map(|(((initializer, condition), increment), body)| Stmt::For {
            initializer: initializer.map(Box::new),
            condition,
            increment,
            body: Box::new(body),
        })
        .map_with(to_spanned)
        .labelled("for loop")
        .as_context();

    let break_ = just(Token::Break)
        .then(just(Token::Semicolon))
        .to(Stmt::Break)
        .map_with(to_spanned)
        .labelled("break")
        .as_context();

    let continue_ = just(Token::Continue)
        .then(just(Token::Semicolon))
        .to(Stmt::Continue)
        .map_with(to_spanned)
        .labelled("continue")
        .as_context();

    let fun = fun_decl(stmt_parser.clone()).map(|spanned_fun| spanned_fun.map(Stmt::FunDecl));

    let class =
        class_decl(stmt_parser.clone()).map(|spanned_class| spanned_class.map(Stmt::ClassDecl));

    let return_ = just(Token::Return)
        .ignore_then(expr_parser.clone())
        .then_ignore(just(Token::Semicolon))
        .map(Stmt::Return)
        .map_with(to_spanned)
        .labelled("return statement")
        .as_context();

    print
        .or(var_decl(expr_parser.clone()))
        .or(class)
        .or(block)
        .or(if_)
        .or(while_)
        .or(for_)
        .or(break_)
        .or(continue_)
        .or(fun)
        .or(return_)
        .or(expr_stmt(expr_parser.clone()))
}

pub(crate) fn expr_or_program_parser<'a, I>() -> impl Parser<'a, I, ProgramOrExpr, Err<'a>>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let mut stmt = Recursive::declare();
    let mut expr = Recursive::declare();

    stmt.define(stmt_parser(stmt.clone(), expr.clone()));
    expr.define(expr_parser(stmt.clone(), expr.clone()));

    expr.map(ProgramOrExpr::Expr)
        .then_ignore(end())
        .or(program_parser().map(ProgramOrExpr::Program))
}

pub(crate) fn program_parser<'a, I>() -> impl Parser<'a, I, Vec<Spanned<Stmt>>, Err<'a>>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    let mut stmt = Recursive::declare();
    let mut expr = Recursive::declare();

    stmt.define(stmt_parser(stmt.clone(), expr.clone()));
    expr.define(expr_parser(stmt.clone(), expr.clone()));

    stmt.repeated().collect().then_ignore(end())
}
