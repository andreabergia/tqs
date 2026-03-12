use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;

use super::list;

#[derive(Debug, Parser)]
#[command(about = "List tasks in the inbox queue")]
pub struct Inbox;

pub fn handle_inbox(_: Inbox, root: Option<PathBuf>) -> Result<(), AppError> {
    list::print_queue(list::QueueSelection::Inbox, root)
}

#[cfg(test)]
mod tests {
    use super::Inbox;
    use clap::Parser;

    #[test]
    fn parses_inbox_command() {
        Inbox::parse_from(["inbox"]);
    }
}
