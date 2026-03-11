use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::app::app_error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEditor {
    pub command: String,
    pub program: String,
    pub args: Vec<String>,
}

impl ResolvedEditor {
    pub fn resolve() -> Result<Self, AppError> {
        let command = env::var("VISUAL")
            .or_else(|_| env::var("EDITOR"))
            .unwrap_or_else(|_| "vi".to_string());

        let mut parts = shell_words::split(&command).map_err(|error| {
            AppError::message(format!("invalid editor command '{}': {}", command, error))
        })?;

        if parts.is_empty() {
            return Err(AppError::message("editor command is empty"));
        }

        let program = parts.remove(0);
        Ok(Self {
            command,
            program,
            args: parts,
        })
    }

    pub fn executable_path(&self) -> Option<PathBuf> {
        find_executable(&self.program)
    }
}

fn find_executable(program: &str) -> Option<PathBuf> {
    let program_path = Path::new(program);
    if program_path.components().count() > 1 {
        return is_executable(program_path).then(|| program_path.to_path_buf());
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        true
    }
}

pub fn format_program_path(path: &Path) -> String {
    path.as_os_str().to_string_lossy().into_owned()
}

pub fn format_program_name(program: impl AsRef<OsStr>) -> String {
    program.as_ref().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::ResolvedEditor;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn visual_overrides_editor() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "nvim --clean");
            std::env::set_var("EDITOR", "vim");
        }

        let editor = ResolvedEditor::resolve().expect("editor should resolve");
        assert_eq!(editor.command, "nvim --clean");
        assert_eq!(editor.program, "nvim");
        assert_eq!(editor.args, vec!["--clean"]);

        unsafe {
            std::env::remove_var("VISUAL");
            std::env::remove_var("EDITOR");
        }
    }

    #[test]
    fn editor_is_used_when_visual_is_unset() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::remove_var("VISUAL");
            std::env::set_var("EDITOR", "hx");
        }

        let editor = ResolvedEditor::resolve().expect("editor should resolve");
        assert_eq!(editor.command, "hx");
        assert_eq!(editor.program, "hx");
        assert!(editor.args.is_empty());

        unsafe {
            std::env::remove_var("EDITOR");
        }
    }

    #[test]
    fn falls_back_to_vi() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::remove_var("VISUAL");
            std::env::remove_var("EDITOR");
        }

        let editor = ResolvedEditor::resolve().expect("editor should resolve");
        assert_eq!(editor.command, "vi");
        assert_eq!(editor.program, "vi");
        assert!(editor.args.is_empty());
    }

    #[test]
    fn parses_shell_quoted_command() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "code --wait \"notes.md\"");
            std::env::remove_var("EDITOR");
        }

        let editor = ResolvedEditor::resolve().expect("editor should resolve");
        assert_eq!(editor.program, "code");
        assert_eq!(editor.args, vec!["--wait", "notes.md"]);

        unsafe {
            std::env::remove_var("VISUAL");
        }
    }

    #[test]
    fn rejects_invalid_shell_syntax() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "\"unterminated");
            std::env::remove_var("EDITOR");
        }

        let error = ResolvedEditor::resolve().expect_err("resolution should fail");
        assert!(
            error
                .to_string()
                .contains("invalid editor command '\"unterminated'")
        );

        unsafe {
            std::env::remove_var("VISUAL");
        }
    }
}
