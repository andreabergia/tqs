use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Usage(String),
}

impl AppError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Message(_) => 1,
            Self::Usage(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn usage_error_maps_to_exit_code_2() {
        let error = AppError::usage("bad args");
        assert_eq!(error.exit_code(), 2);
    }
}
