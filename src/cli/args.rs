use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tqs", version, about = "Terminal task queue")]
pub struct Cli {
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Create {
        summary: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    List {
        keywords: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        closed: bool,
        #[arg(long)]
        verbose: bool,
    },
    Complete {
        id: Option<String>,
    },
    Reopen {
        id: Option<String>,
    },
    Info {
        id: Option<String>,
    },
    Delete {
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parses_list_command() {
        let cli = Cli::parse_from(["tqs", "list", "foo", "bar"]);

        match cli.command {
            Some(Command::List { keywords, .. }) => assert_eq!(keywords, vec!["foo", "bar"]),
            _ => panic!("expected list command"),
        }
    }
}
