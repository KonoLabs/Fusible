//! Concurrency limits: a counting semaphore and per-key locks.

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

const CONVERSIONS: usize = 2;
const HANDLERS: usize = 64;

pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A counting semaphore; a [`Permit`] gives its slot back when dropped.
pub(super) struct Semaphore {
    free: Mutex<usize>,
    wake: Condvar,
}

/// A held slot of a [`Semaphore`].
pub(super) struct Permit(Arc<Semaphore>);

impl Semaphore {
    pub(super) fn new(slots: usize) -> Arc<Self> {
        Arc::new(Self {
            free: Mutex::new(slots),
            wake: Condvar::new(),
        })
    }

    /// Waits for a free slot.
    pub(super) fn acquire(self: &Arc<Self>) -> Permit {
        let mut free = lock(&self.free);
        while *free == 0 {
            free = self.wake.wait(free).unwrap_or_else(PoisonError::into_inner);
        }
        *free -= 1;
        Permit(self.clone())
    }

    /// Takes a free slot, or `None` when all are in use.
    pub(super) fn try_acquire(self: &Arc<Self>) -> Option<Permit> {
        let mut free = lock(&self.free);
        (*free > 0).then(|| {
            *free -= 1;
            Permit(self.clone())
        })
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        *lock(&self.0.free) += 1;
        self.0.wake.notify_one();
    }
}

/// The slots shared by every body conversion and preview build.
pub(super) fn conversions() -> &'static Arc<Semaphore> {
    static SLOTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SLOTS.get_or_init(|| Semaphore::new(CONVERSIONS))
}

/// The slots of requests handled at once; more are answered with 503.
pub(super) fn handlers() -> &'static Arc<Semaphore> {
    static SLOTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SLOTS.get_or_init(|| Semaphore::new(HANDLERS))
}

/// Returns the lock shared by every caller that asks for the same `key`.
pub(super) fn key_lock(key: &str) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    let mut locks = lock(LOCKS.get_or_init(Mutex::default));
    locks.retain(|_, held| Arc::strong_count(held) > 1);
    locks.entry(key.to_string()).or_default().clone()
}
