use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Usage(String),
    #[error("task not found: {id}")]
    NotFound { id: String },
    #[error("interactive input requires a TTY")]
    NoTty,
    #[error("invalid task file {path}: {reason}")]
    InvalidTaskFile { path: String, reason: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("format error: {0}")]
    Format(#[from] crate::storage::format::FormatError),
    #[error("dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
}

impl AppError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound { id: id.into() }
    }

    pub fn invalid_task_file(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidTaskFile {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Message(_)
            | Self::NotFound { .. }
            | Self::NoTty
            | Self::InvalidTaskFile { .. }
            | Self::Io(_)
            | Self::Yaml(_)
            | Self::Format(_)
            | Self::Dialoguer(_) => 1,
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

    #[test]
    fn operational_errors_map_to_exit_code_1() {
        assert_eq!(AppError::message("oops").exit_code(), 1);
        assert_eq!(AppError::not_found("abc").exit_code(), 1);
        assert_eq!(AppError::NoTty.exit_code(), 1);
        assert_eq!(
            AppError::invalid_task_file("a.md", "bad yaml").exit_code(),
            1
        );
    }
}
