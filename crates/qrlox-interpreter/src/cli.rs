use std::path::PathBuf;

use chumsky::Parser as _;
use clap::Parser;
use logos::Logos;

use crate::{
    ast,
    interpreter::Interpreter,
    lex::Token,
    parse,
    repl::{create_error_report, repl},
};

#[derive(Parser, Clone)]
#[command(version, author, about)]
struct CliArgs {
    #[arg()]
    /// The file to run
    file: Option<PathBuf>,
}

pub fn cli() {
    let args = CliArgs::parse();

    if let Some(file) = args.file {
        let content = std::fs::read_to_string(file).expect("Could not read file");

        let stream = Token::lexer(&content)
            .spanned()
            .map(|(tok, range)| match tok {
                Ok(tok) => (tok, ast::Span::new(range)),
                Err(()) => (Token::Error, ast::Span::new(range)),
            });
        let n_chars = content.chars().count();
        let stream = chumsky::Stream::from_iter(ast::Span::new(n_chars..n_chars), stream);
        let (ast, errors) = parse::program_parser().parse_recovery(stream);
        let ast = match (ast, &errors[..]) {
            (Some(ast), []) => ast,
            (_, errors) => {
                for err in errors.iter().map(|e| parse::create_report(e)) {
                    err.eprint(ariadne::Source::from(&content)).unwrap()
                }
                std::process::exit(1);
            }
        };

        let mut interpreter = Interpreter::new();
        let res = interpreter.exec_program(&ast);

        match res {
            Ok(()) => {}
            Err(err) => create_error_report(&err)
                .eprint(ariadne::Source::from(&content))
                .unwrap(),
        }
    } else {
        repl();
    }
}
