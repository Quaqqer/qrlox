pub mod ast;
mod parse;
mod token;

pub use ast::{spanned, Binop, Expr, Spanned, Stmt};

use chumsky::{error::Simple, Parser};
use logos::Logos as _;
use token::Token;

#[allow(clippy::type_complexity)]
fn token_stream<'a>(
    source: &'a str,
) -> chumsky::Stream<
    'a,
    Token<'a>,
    ast::Span,
    std::iter::Map<
        logos::SpannedIter<'a, Token<'a>>,
        impl FnMut((Result<Token<'a>, ()>, std::ops::Range<usize>)) -> (Token<'a>, ast::Span),
    >,
> {
    let lexer_stream = Token::lexer(source)
        .spanned()
        .map(|(tok, range)| match tok {
            Ok(tok) => (tok, ast::Span::new(range)),
            Err(()) => (Token::Error, ast::Span::new(range)),
        });
    let n_chars = source.len();
    chumsky::Stream::from_iter(ast::Span::new(n_chars..n_chars), lexer_stream)
}

pub fn parse_expr_or_program<'a, 'b>(
    source: &'a str,
    ariadne_config: &'a ariadne::Config,
) -> (Option<ProgramOrExpr<'a>>, Vec<ariadne::Report<'b>>) {
    let stream = token_stream(source);
    let (ast, errors) = parse::expr_or_program_parser().parse_recovery(stream);
    let errors = errors
        .iter()
        .map(|err| error_report(err, ariadne_config))
        .collect();
    (ast, errors)
}

pub fn parse_program<'a, 'b>(
    source: &'a str,
    ariadne_config: &'a ariadne::Config,
) -> (Option<ProgramOrExpr<'a>>, Vec<ariadne::Report<'b>>) {
    let stream = token_stream(source);
    let (ast, errors) = parse::expr_or_program_parser().parse_recovery(stream);
    let errors = errors
        .iter()
        .map(|err| error_report(err, ariadne_config))
        .collect();
    (ast, errors)
}

pub enum ProgramOrExpr<'a> {
    Program(Vec<Spanned<Stmt<'a>>>),
    Expr(Spanned<Expr<'a>>),
}

pub fn error_report<'a>(
    err: &Simple<Token<'_>, ast::Span>,
    ariadne_config: &ariadne::Config,
) -> ariadne::Report<'a> {
    ariadne::Report::build(ariadne::ReportKind::Error, err.span().range().clone())
        .with_config(ariadne_config.with_index_type(ariadne::IndexType::Byte))
        .with_message(err.to_string())
        .with_label(
            ariadne::Label::new(err.span().range().clone())
                .with_message(err.reason().to_string())
                .with_color(ariadne::Color::Red),
        )
        .finish()
}
