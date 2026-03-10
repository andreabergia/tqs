use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::matches_query;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Find {
    pub query: String,
}

pub fn handle_find(
    Find { query }: Find,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global)?;
    let matches = repo
        .scan_all()?
        .into_iter()
        .filter(|stored| matches_query(&stored.task, &query))
        .collect::<Vec<_>>();

    output::print_search_results(&matches);
    Ok(())
}
