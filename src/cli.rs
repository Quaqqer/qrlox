use std::path::PathBuf;

use clap::Parser;

use crate::repl::repl;

#[derive(Parser, Clone)]
#[command(version, author, about)]
struct CliArgs {
    #[arg()]
    /// The file to run
    file: Option<PathBuf>,
}

pub fn cli() {
    let args = CliArgs::parse();

    if args.file.is_some() {
        unimplemented!("Can't run qrlox with a file yet")
    }

    repl();
}
