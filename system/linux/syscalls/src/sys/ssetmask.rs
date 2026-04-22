use celer_system_linux_ctypes::OldSigsetT;

use crate::arch::current::{Sysno, syscall1};

/// Replace the caller's legacy blocked-signal mask word.
///
/// This is the obsolete i386 `ssetmask` compatibility syscall superseded by
/// `sigprocmask`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 0.10 stripped only `SIGKILL`; Linux 1.0 and
///   current kernels strip both `SIGKILL` and `SIGSTOP` before installing the
///   new mask
/// - Availability: this wrapper targets the i386 ABI; current kernels also
///   retain `ssetmask` on other legacy arch ABIs, subject to
///   `CONFIG_SGETMASK_SYSCALL`
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On supported kernels, installs `newmask` as the low 32-bit blocked-signal
///   mask word for the calling task and returns the previous mask word.
/// - Current kernels sanitize `newmask` through `set_current_blocked()`, which
///   removes `SIGKILL` and `SIGSTOP` before storing it.
/// - This wrapper exposes the raw register value as [`OldSigsetT`].
/// - Current kernels built without `CONFIG_SGETMASK_SYSCALL` route the call to
///   `sys_ni_syscall`, so the raw return bits encode `-ENOSYS` instead of a
///   previous signal mask.
///
/// # Errors
/// - `ENOSYS`: current kernels built without `CONFIG_SGETMASK_SYSCALL`
///   dispatch `ssetmask` to `sys_ni_syscall`; because this wrapper returns the
///   raw previous-mask word, callers observe the raw register bits for
///   `-ENOSYS`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigprocmask.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4787)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4789)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n88)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.10#n20)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n78)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn ssetmask(newmask: OldSigsetT) -> OldSigsetT {
    // SAFETY: `ssetmask` takes a scalar mask word and has no Rust-side safety
    // preconditions.
    unsafe { syscall1(Sysno::Ssetmask, newmask as isize) as OldSigsetT }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use celer_system_linux_ctypes::OldSigsetT;

    use super::ssetmask;

    const ENOSYS_BITS: OldSigsetT = (-38_i32) as OldSigsetT;
    const SIGKILL: OldSigsetT = 9;
    const SIGUSR1: OldSigsetT = 10;
    const SIGUSR2: OldSigsetT = 12;
    const SIGSTOP: OldSigsetT = 19;

    static SSETMASK_LOCK: Mutex<()> = Mutex::new(());

    struct RestoreMask(OldSigsetT);

    impl Drop for RestoreMask {
        fn drop(&mut self) {
            let _ = ssetmask(self.0);
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn ssetmask_original_mask() -> Option<OldSigsetT> {
        let original = ssetmask(0);
        if original == ENOSYS_BITS {
            None
        } else {
            Some(original)
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_ssetmask_strips_sigkill_and_sigstop() {
        let _guard = SSETMASK_LOCK.lock().unwrap();

        let Some(original) = ssetmask_original_mask() else {
            return;
        };
        let _restore = RestoreMask(original);

        let requested = (1 as OldSigsetT) << 31
            | ((1 as OldSigsetT) << (SIGUSR1 - 1))
            | ((1 as OldSigsetT) << (SIGKILL - 1))
            | ((1 as OldSigsetT) << (SIGSTOP - 1));

        let previous = ssetmask(requested);
        assert_eq!(previous, 0, "ssetmask should return the previous mask");

        let expected =
            ((1 as OldSigsetT) << 31) | ((1 as OldSigsetT) << (SIGUSR1 - 1));
        let replacement =
            ((1 as OldSigsetT) << 30) | ((1 as OldSigsetT) << (SIGUSR2 - 1));
        let previous = ssetmask(replacement);
        assert_eq!(
            previous, expected,
            "ssetmask should return the previous effective mask"
        );

        let effective = ssetmask(0);
        assert_eq!(
            effective, replacement,
            "ssetmask should install the replacement mask unchanged"
        );
    }
}
