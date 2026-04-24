use std::sync::{Mutex, MutexGuard};

#[cfg(target_arch = "aarch64")]
pub(crate) use libc::signal;
pub(crate) use libc::{_exit, fork, getpgrp, open, pause, pipe, waitpid};

static PROCESS_GLOBAL_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn process_global_state_guard() -> MutexGuard<'static, ()> {
    PROCESS_GLOBAL_TEST_LOCK.lock().unwrap()
}
