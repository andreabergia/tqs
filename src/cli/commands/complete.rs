use clap::Parser;

use crate::app::app_error::AppError;

#[derive(Debug, Parser)]
pub struct Complete {
    pub id: Option<String>,
}

pub fn handle_complete(Complete { .. }: Complete) -> Result<(), AppError> {
    println!("complete is not implemented yet");
    Ok(())
}
