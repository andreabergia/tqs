use clap::Parser;

use crate::cli::{args::Cli, handlers};

pub fn run() -> i32 {
    let cli = Cli::parse();

    match handlers::handle(cli) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code()
        }
    }
}
