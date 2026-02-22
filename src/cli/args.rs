use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::commands::{Complete, Create, Delete, Info, List, Move, Reopen};

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
    Create(Create),
    List(List),
    Complete(Complete),
    Reopen(Reopen),
    Info(Info),
    Delete(Delete),
    Move(Move),
}
