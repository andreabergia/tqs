use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Config;

pub fn handle_config(_: Config, root: Option<PathBuf>, global: bool) -> Result<(), AppError> {
    let resolved = helpers::resolve_config(root, global)?;
    output::print_config(&resolved);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Config;
    use clap::Parser;

    #[test]
    fn parses_config_command() {
        Config::parse_from(["config"]);
    }
}
