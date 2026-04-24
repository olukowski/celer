use std::sync::{Mutex, MutexGuard};

use celer_system_linux_ctypes::{Char, Int, PidT, UnsignedInt};

static PROCESS_GLOBAL_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn process_global_state_guard() -> MutexGuard<'static, ()> {
    PROCESS_GLOBAL_TEST_LOCK.lock().unwrap()
}

unsafe extern "C" {
    pub(crate) fn _exit(status: Int) -> !;
    pub(crate) fn fork() -> Int;
    pub(crate) fn getpgrp() -> PidT;
    pub(crate) fn open(path: *const Char, flags: Int, mode: UnsignedInt)
    -> Int;
    pub(crate) fn pause() -> Int;
    pub(crate) fn pipe(fds: *mut Int) -> Int;
    pub(crate) fn signal(sig: Int, handler: *const ()) -> *const ();
    pub(crate) fn waitpid(pid: Int, status: *mut Int, options: Int) -> Int;
}
