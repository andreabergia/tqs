use clap::Parser;

use crate::app::app_error::AppError;

#[derive(Debug, Parser)]
pub struct Info {
    pub id: Option<String>,
}

pub fn handle_info(Info { .. }: Info) -> Result<(), AppError> {
    println!("info is not implemented yet");
    Ok(())
}
