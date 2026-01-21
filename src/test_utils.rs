use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Debug)]
pub struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    values: HashMap<String, Option<String>>,
}

impl EnvGuard {
    pub fn new(keys: &[&str]) -> Self {
        let lock = ENV_MUTEX.lock().expect("Failed to lock env mutex");
        let values = keys
            .iter()
            .map(|key| ((*key).to_string(), std::env::var(*key).ok()))
            .collect();
        Self {
            _lock: lock,
            values,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.values.drain() {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}
