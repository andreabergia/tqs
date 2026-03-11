use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    sync::{Mutex, MutexGuard, OnceLock},
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) struct LockedEnv {
    _guard: MutexGuard<'static, ()>,
    original_values: HashMap<String, Option<OsString>>,
}

impl LockedEnv {
    pub(crate) fn new(names: &[&str]) -> Self {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let original_values = names
            .iter()
            .map(|name| ((*name).to_string(), env::var_os(name)))
            .collect();

        Self {
            _guard: guard,
            original_values,
        }
    }

    pub(crate) fn set(&mut self, name: &str, value: impl Into<OsString>) {
        unsafe {
            env::set_var(name, value.into());
        }
    }

    pub(crate) fn remove(&mut self, name: &str) {
        unsafe {
            env::remove_var(name);
        }
    }
}

impl Drop for LockedEnv {
    fn drop(&mut self) {
        for (name, value) in &self.original_values {
            match value {
                Some(value) => unsafe {
                    env::set_var(name, value);
                },
                None => unsafe {
                    env::remove_var(name);
                },
            }
        }
    }
}
