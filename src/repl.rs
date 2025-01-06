use crate::{
    ast::{self, Span},
    interpreter::{self, InterpretorCtx},
    lex::Token,
    parse::{create_report, expr_or_stmt_parser},
    ARIADNE_CONFIG,
};
use chumsky::{prelude::end, Parser};
use logos::Logos;

pub fn repl() {
    let appdirs = platform_dirs::AppDirs::new(Some("twlox"), true)
        .expect("Failed to load app directories for platform");

    let mut rl = rustyline::DefaultEditor::new().expect("Failed to create readline");

    let hist_dir = appdirs.state_dir;
    let hist_file = hist_dir.join("repl_history.txt");

    // Attempt to load history
    let _ = rl.load_history(&hist_file);

    let mut ctx = InterpretorCtx::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // Add line to history
                if let Err(e) = rl.add_history_entry(line.as_str()) {
                    eprintln!("Failed to add line to history: {}", e);
                };

                // Parse
                let stream = Token::lexer(&line).spanned().map(|(tok, range)| match tok {
                    Ok(tok) => (tok, ast::Span::new(range)),
                    Err(()) => (Token::Error, ast::Span::new(range)),
                });
                let n_chars = line.chars().count();
                let stream = chumsky::Stream::from_iter(Span::new(n_chars..n_chars), stream);
                let (ast, errors) = expr_or_stmt_parser()
                    .then_ignore(end())
                    .parse_recovery(stream);

                // Check if parsing was correct
                let ast = match (ast, &errors[..]) {
                    (Some(ast), []) => ast,
                    (_, errors) => {
                        for err in errors.iter().map(|e| create_report(e)) {
                            err.eprint(ariadne::Source::from(&line)).unwrap()
                        }
                        continue;
                    }
                };

                // Evaluate parsed ast
                match ast {
                    crate::parse::StmtOrExpr::Stmt(stmt) => match ctx.exec_stmt(&stmt) {
                        Ok(()) => {}
                        Err(err) => create_error_report(&err)
                            .eprint(ariadne::Source::from(&line))
                            .unwrap(),
                    },
                    crate::parse::StmtOrExpr::Expr(expr) => match ctx.eval_expr(&expr) {
                        Ok(value) => println!("{}", value.repr()),
                        Err(err) => create_error_report(&err)
                            .eprint(ariadne::Source::from(&line))
                            .unwrap(),
                    },
                }
            }
            Err(
                rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof,
            ) => {
                println!("Quitting...");
                break;
            }
            Err(err) => {
                eprintln!("Got error when trying to read line: {}", err);
            }
        }
    }

    std::fs::create_dir_all(&hist_dir).expect("Failed to create directory for history");
    rl.save_history(&hist_file).expect("Failed to save history");
}

pub fn create_error_report(err: &interpreter::Error) -> ariadne::Report {
    ariadne::Report::build(ariadne::ReportKind::Error, err.span.range.clone())
        .with_config(ARIADNE_CONFIG)
        .with_label(
            ariadne::Label::new(err.span.range.clone())
                .with_message(err.message.clone())
                .with_color(ariadne::Color::Red),
        )
        .finish()
}
