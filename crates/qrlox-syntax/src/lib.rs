pub mod ast;
mod parse;
mod token;

use ast::Span;
pub use ast::{spanned, Binop, Expr, Spanned, Stmt};

use chumsky::{
    error::Rich,
    input::{Input as _, Stream},
    Parser,
};
use logos::Logos as _;
use token::Token;

#[allow(clippy::type_complexity)]
fn token_stream<'a>(
    source: &'a str,
) -> chumsky::input::MappedInput<
    Token<'a>,
    Span,
    Stream<
        std::iter::Map<
            logos::SpannedIter<'a, Token<'a>>,
            impl FnMut((Result<Token<'a>, ()>, std::ops::Range<usize>)) -> (Token<'a>, Span),
        >,
    >,
    impl Fn((Token<'a>, Span)) -> (Token<'a>, Span),
> {
    let token_iter = Token::lexer(source)
        .spanned()
        .map(|(tok, range)| match tok {
            Ok(tok) => (tok, ast::Span::new(range)),
            Err(()) => (Token::Error, ast::Span::new(range)),
        });
    Stream::from_iter(token_iter).map(Span::new(0..source.len()), |(t, s)| (t, s))
}

pub fn parse_expr_or_program<'a, 'b>(
    source: &'a str,
    ariadne_config: &'a ariadne::Config,
) -> (Option<ProgramOrExpr>, Vec<ariadne::Report<'b>>) {
    let stream = token_stream(source);
    let (ast, errors) = parse::expr_or_program_parser()
        .parse(stream)
        .into_output_errors();
    let errors = errors
        .iter()
        .map(|err| error_report(err, ariadne_config))
        .collect();
    (ast, errors)
}

pub fn parse_program<'a, 'b>(
    source: &'a str,
    ariadne_config: &'a ariadne::Config,
) -> (Option<Vec<Spanned<Stmt>>>, Vec<ariadne::Report<'b>>) {
    let stream = token_stream(source);
    let (ast, errors) = parse::program_parser().parse(stream).into_output_errors();
    let errors = errors
        .iter()
        .map(|err| error_report(err, ariadne_config))
        .collect();
    (ast, errors)
}

pub enum ProgramOrExpr {
    Program(Vec<Spanned<Stmt>>),
    Expr(Spanned<Expr>),
}

pub fn error_report<'a>(
    err: &Rich<Token<'_>, ast::Span>,
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
