use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;

use super::list;

#[derive(Debug, Parser)]
pub struct Now;

pub fn handle_now(_: Now, root: Option<PathBuf>) -> Result<(), AppError> {
    list::print_queue(list::QueueSelection::Now, root)
}

#[cfg(test)]
mod tests {
    use super::Now;
    use clap::Parser;

    #[test]
    fn parses_now_command() {
        Now::parse_from(["now"]);
    }
}
