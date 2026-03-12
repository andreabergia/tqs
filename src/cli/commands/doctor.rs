use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::io::output;
use crate::storage::doctor;

#[derive(Debug, Parser)]
pub struct Doctor;

pub fn handle_doctor(_: Doctor, root: Option<PathBuf>) -> Result<(), AppError> {
    let resolved = helpers::resolve_config(root)?;
    let report = doctor::run(&resolved)?;
    output::print_doctor_report(&report);

    if report.has_errors() {
        return Err(AppError::message(format!(
            "doctor found {} error(s) and {} warning(s)",
            report.error_count(),
            report.warning_count()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Doctor;
    use clap::Parser;

    #[test]
    fn parses_doctor_command() {
        Doctor::parse_from(["doctor"]);
    }
}
