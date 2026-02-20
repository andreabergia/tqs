use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TaskRepo {
    pub root: PathBuf,
}

impl TaskRepo {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}
