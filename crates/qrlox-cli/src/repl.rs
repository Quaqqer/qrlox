use qrlox_interpreter::Interpreter;
use qrlox_syntax::ProgramOrExpr;

pub fn repl(ariadne_config: &ariadne::Config) {
    let appdirs = platform_dirs::AppDirs::new(Some("twlox"), true)
        .expect("Failed to load app directories for platform");

    let mut rl = rustyline::DefaultEditor::new().expect("Failed to create readline");

    let hist_dir = appdirs.state_dir;
    let hist_file = hist_dir.join("repl_history.txt");

    // Attempt to load history
    let _ = rl.load_history(&hist_file);

    let mut interpreter = Interpreter::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // Add line to history
                if let Err(e) = rl.add_history_entry(line.as_str()) {
                    eprintln!("Failed to add line to history: {}", e);
                };

                // Parse program
                let (ast, errs) = qrlox_syntax::parse_expr_or_program(&line, ariadne_config);
                let ast = match (ast, &errs[..]) {
                    (Some(ast), []) => ast,
                    (_, errs) => {
                        for err in errs {
                            err.eprint(ariadne::Source::from(&line)).unwrap()
                        }
                        continue;
                    }
                };

                let res = match ast {
                    ProgramOrExpr::Program(stmts) => interpreter
                        .exec_program(&stmts, ariadne_config)
                        .map(|_| None),
                    ProgramOrExpr::Expr(expr) => {
                        interpreter.eval_expr(&expr, ariadne_config).map(Some)
                    }
                };

                match res {
                    Ok(Some(v)) => println!("{}", v.repr()),
                    Ok(None) => {}
                    Err(err) => err.eprint(ariadne::Source::from(&line)).unwrap(),
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
