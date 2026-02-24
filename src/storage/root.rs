use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

const TODOS_DIR: &str = "todos";

pub fn resolve_root(explicit_root: Option<PathBuf>, global: bool) -> PathBuf {
    if let Some(root) = explicit_root {
        return root;
    }

    if let Some(root) = env_path("TQS_ROOT") {
        return root;
    }

    if !global {
        let cwd = env::current_dir().ok();
        if let Some(git_root) = cwd.as_deref().and_then(git_root_from) {
            return git_root.join(TODOS_DIR);
        }
    }

    if let Some(xdg_data) = env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    {
        return xdg_data.join("tqs").join(TODOS_DIR);
    }

    if let Some(home) = env_home() {
        return home
            .join(".local")
            .join("share")
            .join("tqs")
            .join(TODOS_DIR);
    }

    env::current_dir()
        .ok()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(TODOS_DIR)
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn env_home() -> Option<PathBuf> {
    env_path("HOME")
}

fn git_root_from(start_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(start_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let root = text.trim();
    if root.is_empty() {
        return None;
    }

    Some(PathBuf::from(root))
}

#[cfg(test)]
mod tests {
    use super::{TODOS_DIR, git_root_from};
    use std::{path::PathBuf, process::Command};
    use tempfile::TempDir;

    fn choose_root(
        explicit_root: Option<PathBuf>,
        env_root: Option<PathBuf>,
        git_root: Option<PathBuf>,
        home_root: Option<PathBuf>,
        cwd: PathBuf,
        global: bool,
    ) -> PathBuf {
        if let Some(root) = explicit_root {
            return root;
        }
        if let Some(root) = env_root {
            return root;
        }
        if !global {
            if let Some(root) = git_root {
                return root.join(TODOS_DIR);
            }
        }
        if let Some(root) = home_root {
            return root
                .join(".local")
                .join("share")
                .join("tqs")
                .join(TODOS_DIR);
        }

        cwd.join(TODOS_DIR)
    }

    #[test]
    fn precedence_prefers_explicit_root() {
        let root = choose_root(
            Some(PathBuf::from("/explicit")),
            Some(PathBuf::from("/env")),
            Some(PathBuf::from("/git")),
            Some(PathBuf::from("/home")),
            PathBuf::from("/cwd"),
            false,
        );

        assert_eq!(root, PathBuf::from("/explicit"));
    }

    #[test]
    fn precedence_uses_env_when_no_explicit_root() {
        let root = choose_root(
            None,
            Some(PathBuf::from("/env")),
            Some(PathBuf::from("/git")),
            Some(PathBuf::from("/home")),
            PathBuf::from("/cwd"),
            false,
        );

        assert_eq!(root, PathBuf::from("/env"));
    }

    #[test]
    fn precedence_uses_git_root_before_home() {
        let root = choose_root(
            None,
            None,
            Some(PathBuf::from("/git")),
            Some(PathBuf::from("/home")),
            PathBuf::from("/cwd"),
            false,
        );

        assert_eq!(root, PathBuf::from("/git/todos"));
    }

    #[test]
    fn precedence_uses_home_when_no_git_root() {
        let root = choose_root(
            None,
            None,
            None,
            Some(PathBuf::from("/home/me")),
            PathBuf::from("/cwd"),
            false,
        );

        assert_eq!(root, PathBuf::from("/home/me/.local/share/tqs/todos"));
    }

    #[test]
    fn falls_back_to_cwd_when_nothing_else_available() {
        let root = choose_root(None, None, None, None, PathBuf::from("/cwd"), false);
        assert_eq!(root, PathBuf::from("/cwd/todos"));
    }

    #[test]
    fn global_flag_skips_git_root() {
        let root = choose_root(
            None,
            None,
            Some(PathBuf::from("/git")),
            Some(PathBuf::from("/home/me")),
            PathBuf::from("/cwd"),
            true,
        );

        assert_eq!(root, PathBuf::from("/home/me/.local/share/tqs/todos"));
    }

    #[test]
    fn git_root_is_detected_for_repo_directory() {
        let temp = TempDir::new().expect("temp dir should be created");
        Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(temp.path())
            .status()
            .expect("git init should run");

        let nested = temp.path().join("nested").join("child");
        std::fs::create_dir_all(&nested).expect("nested directories should be created");

        let git_root = git_root_from(&nested).expect("git root should be resolved");
        let expected = temp
            .path()
            .canonicalize()
            .expect("temp path should canonicalize");
        assert_eq!(git_root, expected);
    }
}
