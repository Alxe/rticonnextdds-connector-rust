//! Environment-related test utilities.

/// A helper struct to temporarily set environment variables for a given scope.
/// On drop, it restores the previous value of the environment variable.
///
/// This is useful for test cases where we're creating a Connector that relies on
/// environment variables. It uses a mutex to ensure thread safety.
pub struct EnvDropGuard {
    name: String,
    old_value: Option<String>,
}

impl EnvDropGuard {
    pub fn with_env<F, R>(name: &str, new_value: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let guard = Self::new(name, new_value);
        let result = f();
        drop(guard);
        result
    }

    fn mutex() -> &'static std::sync::Mutex<()> {
        static MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
        &MUTEX
    }

    fn new(name: &str, new_value: &str) -> Self {
        Self {
            name: name.to_string(),
            old_value: {
                let _lock = Self::mutex().lock().unwrap();
                let old_value = std::env::var(name).ok();
                unsafe { std::env::set_var(name, new_value) };

                old_value
            },
        }
    }
}

impl Drop for EnvDropGuard {
    fn drop(&mut self) {
        let _lock = match Self::mutex().lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        match &self.old_value {
            Some(old_value) => unsafe { std::env::set_var(&self.name, old_value) },
            None => unsafe { std::env::remove_var(&self.name) },
        }
    }
}
