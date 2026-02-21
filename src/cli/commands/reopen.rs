use clap::Parser;

use crate::app::app_error::AppError;

#[derive(Debug, Parser)]
pub struct Reopen {
    pub id: Option<String>,
}

pub fn handle_reopen(Reopen { .. }: Reopen) -> Result<(), AppError> {
    println!("reopen is not implemented yet");
    Ok(())
}
