use std::sync::{Mutex, MutexGuard};

static PROCESS_GLOBAL_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn process_global_state_guard() -> MutexGuard<'static, ()> {
    PROCESS_GLOBAL_TEST_LOCK.lock().unwrap()
}
