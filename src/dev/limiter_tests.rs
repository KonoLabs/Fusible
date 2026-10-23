use std::sync::Arc;
use std::time::Duration;

use super::limiter::{key_lock, Semaphore};

#[test]
fn a_semaphore_hands_out_only_its_slots_and_takes_them_back() {
    let slots = Semaphore::new(2);
    let first = slots.acquire();
    let second = slots.try_acquire().expect("a second slot is free");
    assert!(slots.try_acquire().is_none());
    drop(first);
    let waiting = {
        let slots = slots.clone();
        std::thread::spawn(move || drop(slots.acquire()))
    };
    waiting.join().unwrap();
    drop(second);
    assert!(slots.try_acquire().is_some());
}

#[test]
fn a_blocked_acquire_wakes_when_a_slot_is_released() {
    let slots = Semaphore::new(1);
    let held = slots.acquire();
    let waiter = {
        let slots = slots.clone();
        std::thread::spawn(move || drop(slots.acquire()))
    };
    std::thread::sleep(Duration::from_millis(50));
    drop(held);
    waiter.join().unwrap();
    assert!(slots.try_acquire().is_some());
}

#[test]
fn one_key_shares_one_lock_and_stale_locks_are_dropped() {
    let first = key_lock("limiter-test-a");
    assert!(Arc::ptr_eq(&first, &key_lock("limiter-test-a")));
    assert!(!Arc::ptr_eq(&first, &key_lock("limiter-test-b")));
    drop(first);
    let _other = key_lock("limiter-test-c");
    let fresh = key_lock("limiter-test-a");
    assert_eq!(Arc::strong_count(&fresh), 2);
}
