use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::commands::{Add, Config, Doctor, Done, Edit, Find, Inbox, List, Move, Now, Show};

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
    Add(Add),
    List(List),
    Now(Now),
    Inbox(Inbox),
    Move(Move),
    Done(Done),
    Edit(Edit),
    Show(Show),
    Find(Find),
    Config(Config),
    Doctor(Doctor),
}
