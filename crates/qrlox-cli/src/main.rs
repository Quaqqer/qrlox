use std::path::PathBuf;
mod repl;
mod world;

use clap::Parser;
use qrlox_interpreter::Interpreter;
use world::CliWorld;

#[derive(Parser, Clone)]
#[command(version, author, about)]
struct CliArgs {
    #[arg()]
    /// The file to run
    file: Option<PathBuf>,
}

pub fn main() {
    let args = CliArgs::parse();

    let ariadne_config = ariadne::Config::new();

    if let Some(file) = args.file {
        let content = std::fs::read_to_string(file).expect("Could not read file");

        let (ast, errs) = qrlox_syntax::parse_program(&content, &ariadne_config);

        let ast = match (ast, &errs[..]) {
            (Some(ast), []) => ast,
            (_, errs) => {
                for err in errs {
                    err.eprint(ariadne::Source::from(&content)).unwrap()
                }
                std::process::exit(1);
            }
        };

        let mut interpreter = Interpreter::new(CliWorld);
        let res = interpreter.exec_program(&ast, &ariadne_config);

        match res {
            Ok(()) => {}
            Err(err) => err.eprint(ariadne::Source::from(&content)).unwrap(),
        }
    } else {
        repl::repl(&ariadne_config);
    }
}
