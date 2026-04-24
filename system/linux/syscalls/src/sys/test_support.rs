use std::sync::{Mutex, MutexGuard};

pub(crate) use libc::{_exit, fork, getpgrp, open, pause, pipe, waitpid};

static PROCESS_GLOBAL_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn process_global_state_guard() -> MutexGuard<'static, ()> {
    PROCESS_GLOBAL_TEST_LOCK.lock().unwrap()
}
