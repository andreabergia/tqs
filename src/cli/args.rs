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
    Add(Add),
    List(List),
    Move(Move),
    Done(Done),
    Edit(Edit),
    Show(Show),
    Find(Find),
}
