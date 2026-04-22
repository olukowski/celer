use celer_system_linux_ctypes::{Int, OldSigsetT};

use crate::arch::current::{Sysno, syscall1};

/// Return the caller's pending blocked-signal mask word.
///
/// This is the obsolete i386 `sigpending` compatibility syscall superseded by
/// `rt_sigpending`.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 0.12 called `verify_area(set, 4)` but ignored its
///   result and always returned `0`; Linux 1.0 and current kernels return
///   `EFAULT` when the user pointer is not writable
/// - Availability: x86 32-bit exposes the legacy `sys_sigpending` entry point
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, writes one legacy 32-bit signal-mask word to `set`.
/// - The stored value is the bitwise intersection of the caller's blocked and
///   pending signal sets.
/// - Linux 1.0 validated `set` before writing, so its `EFAULT` path left user
///   memory untouched.
/// - Current kernels return `EFAULT` when copying the word to user memory
///   fails and may already have written a prefix of that word before failing.
///
/// # Errors
/// - `EFAULT`: `set` is outside writable user memory.
///
/// # Safety
/// - `set` must be valid to write one [`OldSigsetT`] for the duration of the
///   syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigpending.2.html)
/// - Current mainline:
///   [v7.0-rc7](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v7.0-rc7#n4543)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n96)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.12#n27)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n82)
pub unsafe fn sigpending(set: *mut OldSigsetT) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Sigpending, set.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::sync::Mutex;

    use celer_system_linux_ctypes::{Int, OldSigsetT, PidT};

    use crate::arch::current::{Sysno, syscall1};
    use crate::sys::{exit, fork, getpid, kill, ssetmask, waitpid};

    use super::sigpending;

    const SIGUSR1: Int = 10;

    static SIGPENDING_LOCK: Mutex<()> = Mutex::new(());

    extern "C" fn noop_handler(_: Int) {}

    struct RestoreMask(OldSigsetT);

    impl Drop for RestoreMask {
        fn drop(&mut self) {
            let _ = ssetmask(self.0);
        }
    }

    struct RestoreHandler {
        sig: Int,
        old: usize,
    }

    impl Drop for RestoreHandler {
        fn drop(&mut self) {
            let _ = unsafe { crate::sys::signal(self.sig, self.old) };
        }
    }

    fn assert_clean_exit(status: Int) {
        assert_eq!(status & 0x7f, 0, "child should not die from a signal");
        assert_eq!((status >> 8) & 0xff, 0, "child should exit with status 0");
    }

    #[test]
    fn test_sigpending_reports_blocked_pending_signal() {
        let _guard = SIGPENDING_LOCK.lock().unwrap();

        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");

        fn exercise_sigpending_in_child(pid: PidT) {
            if pid == 0 {
                let previous = unsafe {
                    crate::sys::signal(
                        SIGUSR1,
                        noop_handler as *const () as usize,
                    )
                };
                assert!(
                    previous >= 0,
                    "installing temporary handler failed: {previous}"
                );
                let _restore_handler = RestoreHandler {
                    sig: SIGUSR1,
                    old: previous as usize,
                };

                let blocked = (1 as OldSigsetT) << (SIGUSR1 - 1);
                let original_mask = ssetmask(blocked);
                let _restore_mask = RestoreMask(original_mask);

                let rc = kill(getpid(), SIGUSR1);
                assert_eq!(rc, 0, "sending SIGUSR1 to self failed: {rc}");

                let mut pending = 0 as OldSigsetT;
                // SAFETY: `pending` is writable for one `OldSigsetT`.
                let rc = unsafe { sigpending(&raw mut pending) };
                assert_eq!(rc, 0, "sigpending failed: {rc}");
                assert_eq!(
                    pending & blocked,
                    blocked,
                    "SIGUSR1 should be reported as blocked and pending"
                );

                let clear_mask = ssetmask(original_mask);
                assert_eq!(
                    clear_mask, blocked,
                    "unblocking should return the blocked pending mask"
                );

                exit(0);
            }
        }

        exercise_sigpending_in_child(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_clean_exit(status);
    }

    #[test]
    fn test_sigpending_faults_on_invalid_pointer() {
        // SAFETY: this intentionally passes an invalid kernel ABI pointer to
        // verify the syscall reports `EFAULT`.
        let rc = unsafe { sigpending(usize::MAX as *mut OldSigsetT) };

        assert_eq!(rc, -14, "expected EFAULT for bad set pointer, got {rc}");
    }

    #[test]
    fn test_sigpending_matches_raw_syscall_bits() {
        let mut pending = 0 as OldSigsetT;
        // SAFETY: `pending` is writable for one `OldSigsetT`.
        let wrapped = unsafe { sigpending(&raw mut pending) };
        assert_eq!(wrapped, 0, "sigpending failed: {wrapped}");

        let mut raw_pending = 0 as OldSigsetT;
        // SAFETY: `raw_pending` is writable for one `OldSigsetT`.
        let raw = unsafe {
            syscall1(Sysno::Sigpending, (&raw mut raw_pending).addr() as isize)
        };
        assert_eq!(raw, 0, "raw sigpending failed: {raw}");
        assert_eq!(
            pending, raw_pending,
            "sigpending should match the raw syscall output"
        );
    }
}
