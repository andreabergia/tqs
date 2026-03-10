use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::commands::{Add, Done, Edit, Find, List, Move, Show};

#[derive(Debug, Parser)]
#[command(name = "tqs", version, about = "Terminal task queue")]
pub struct Cli {
    #[arg(short = 'g', long, global = true, conflicts_with = "root")]
    pub global: bool,

    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(visible_aliases = ["create", "new"])]
    Add(Add),
    #[command(visible_alias = "ls")]
    List(List),
    #[command(visible_aliases = ["mv", "rename"])]
    Move(Move),
    #[command(visible_aliases = ["complete", "finish", "close"])]
    Done(Done),
    #[command(visible_alias = "modify")]
    Edit(Edit),
    #[command(visible_aliases = ["info", "view"])]
    Show(Show),
    #[command(visible_alias = "search")]
    Find(Find),
}
