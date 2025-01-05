use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, author, about)]
struct CliArgs {
    #[arg()]
    /// The file to run
    file: Option<PathBuf>,
}

fn main() {
    let args = CliArgs::parse();

    repl();
}

fn repl() {
    let appdirs = platform_dirs::AppDirs::new(Some("twlox"), true)
        .expect("Failed to load app directories for platform");

    let mut rl = rustyline::DefaultEditor::new().expect("Failed to create readline");

    let hist_dir = appdirs.state_dir;
    let hist_file = hist_dir.join("repl_history.txt");

    // Attempt to load history
    let load_res = rl.load_history(&hist_file);

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if let Err(e) = rl.add_history_entry(line.as_str()) {
                    eprintln!("Failed to add line to history: {}", e);
                };

                println!("Read: {}", line);
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
