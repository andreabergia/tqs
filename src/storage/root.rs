use std::path::PathBuf;

pub fn resolve_root(explicit_root: Option<PathBuf>) -> PathBuf {
    explicit_root.unwrap_or_else(|| PathBuf::from("todos"))
}
