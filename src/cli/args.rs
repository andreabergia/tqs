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
    #[command(visible_aliases = ["new", "add"])]
    Create(Create),
    #[command(visible_alias = "ls")]
    List(List),
    #[command(visible_aliases = ["done", "finish", "close"])]
    Complete(Complete),
    #[command(visible_alias = "open")]
    Reopen(Reopen),
    #[command(visible_aliases = ["show", "view"])]
    Info(Info),
    #[command(visible_aliases = ["remove", "rm", "del"])]
    Delete(Delete),
    #[command(visible_aliases = ["rename", "mv"])]
    Move(Move),
}
