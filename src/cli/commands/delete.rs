use clap::Parser;

use crate::app::app_error::AppError;

#[derive(Debug, Parser)]
pub struct Delete {
    pub id: String,
}

pub fn handle_delete(Delete { .. }: Delete) -> Result<(), AppError> {
    println!("delete is not implemented yet");
    Ok(())
}
