use std::sync::{Mutex, MutexGuard, PoisonError};

/// A value a fake keeps and a test sets or inspects, safe to share between threads.
#[derive(Debug, Default)]
pub struct Shared<T>(Mutex<T>);

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    pub fn set(&self, value: T) {
        *self.lock() = value;
    }

    pub fn update<R>(&self, change: impl FnOnce(&mut T) -> R) -> R {
        change(&mut self.lock())
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T: Clone> Shared<T> {
    pub fn get(&self) -> T {
        self.lock().clone()
    }
}
