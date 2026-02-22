use clap::Parser;

use crate::cli::{args::Cli, fuzzy, handlers};

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    let expanded_args = fuzzy::expand_command(args);

    let cli = Cli::try_parse_from(expanded_args.iter()).unwrap_or_else(|e| {
        e.exit();
    });

    match handlers::handle(cli) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code()
        }
    }
}
