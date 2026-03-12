use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::config;

#[derive(Debug, Parser)]
#[command(about = "Show effective configuration and setup help")]
pub struct Config;

pub fn handle_config(_: Config, root: Option<PathBuf>) -> Result<(), AppError> {
    let inspection = config::inspect(root)?;
    output::print_config_inspection(&inspection);
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
