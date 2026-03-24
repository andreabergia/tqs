use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::commands::{
    Add, Config, Delete, Doctor, Done, Edit, Find, Inbox, List, Move, Now, Show, Start, Triage,
};

const TOP_LEVEL_HELP: &str = "\
Task Commands:
  add     Add a task
  list    List tasks
  find    Find tasks by text
  show    Show task details

Workflow Commands:
  now     List tasks in the now queue
  inbox   List tasks in the inbox queue
  start   Move a task to the now queue
  move    Move a task to a different queue
  done    Mark a task as done
  delete  Delete a task permanently
  edit    Edit a task
  triage  Triage inbox tasks interactively

Setup Commands:
  config  Show effective configuration and setup help
  doctor  Check configuration and task storage health

Help:
  help    Print this message or the help of the given subcommand(s)
";

#[derive(Debug, Parser)]
#[command(
    name = "tqs",
    version,
    about = "Terminal task queue",
    help_template = "{about-with-newline}\n{usage-heading} {usage}\n\nOptions:\n{options}{after-help}",
    after_help = TOP_LEVEL_HELP
)]
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
    Start(Start),
    Delete(Delete),
    Done(Done),
    Edit(Edit),
    Show(Show),
    Find(Find),
    Config(Config),
    Triage(Triage),
    Doctor(Doctor),
}
