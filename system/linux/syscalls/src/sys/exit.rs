use celer_system_linux_ctypes::Int;

use core::hint::unreachable_unchecked;

use crate::arch::current::{Sysno, syscall1};

/// Terminates the calling thread.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present
///
/// # Required privileges
/// - None
///
/// # Behavior
/// - Terminates only the calling **thread**, not the entire process.
/// - This syscall never returns.
///
/// # Errors
/// - Never fails (never returns)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/exit.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/exit.c?h=v6.19#n1077)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/exit.c?h=v6.18.18#n1072)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=1.0#n479)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=0.10#n129)
pub fn exit(error_code: Int) -> ! {
    // SAFETY: `exit` is a safe syscall.
    unsafe { syscall1(Sysno::Exit, error_code as isize) };

    // SAFETY: `exit` **never** returns.
    unsafe { unreachable_unchecked() }
}

#[cfg(test)]
mod tests {
    use core::{
        hint::spin_loop,
        sync::atomic::{AtomicBool, Ordering},
    };
    use std::thread;

    use super::exit;

    // NOTE: This test cannot strictly prove that `exit` never returns,
    // but verifies that:
    // - the process remains alive
    // - code after `exit` is not observed in practice
    #[test]
    fn test_exit() {
        static STARTED: AtomicBool = AtomicBool::new(false);
        static AFTER: AtomicBool = AtomicBool::new(false);

        thread::spawn(|| {
            STARTED.store(true, Ordering::Release);
            exit(0);
            #[allow(unreachable_code)]
            AFTER.store(true, Ordering::Relaxed);
        });

        // Wait until thread definitely started
        while !STARTED.load(Ordering::Acquire) {
            spin_loop()
        }

        // Give scheduler a chance
        thread::yield_now();

        assert!(!AFTER.load(Ordering::Relaxed));
    }
}
