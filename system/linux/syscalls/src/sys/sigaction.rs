use celer_system_linux_ctypes::{Int, OldSigaction};

use crate::arch::current::{Sysno, syscall3};

/// Get or set the legacy signal action for `sig`.
///
/// This is the historical i386 `sigaction` syscall ABI, which uses
/// [`OldSigaction`] rather than the newer `rt_sigaction` layout.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 validated handler addresses up front and
///   added `sig` to the installed mask unless `SA_NOMASK` was set; current
///   kernels use the legacy `old_sigaction` ABI and no longer reject handler
///   addresses in the syscall entry path
/// - Availability: x86 32-bit exposes the legacy `sys_sigaction` entry point
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `action == NULL` performs a query-only call.
/// - `oldaction == NULL` skips returning the previous action.
/// - On success, returns `0`.
/// - When `action` is non-null, the kernel rejects attempts to install
///   handlers for `SIGKILL` and `SIGSTOP`.
/// - Linux 1.0 rejected `SIGKILL` and `SIGSTOP` for both query and install
///   calls; current kernels allow query-only calls for those signals.
/// - Current kernels may install the new action and then still return
///   `EFAULT` if writing `oldaction` fails; Linux 1.0 validated both pointers
///   before committing the new action.
///
/// # Errors
/// - `EFAULT`: the kernel could not read `action` or write `oldaction`.
/// - `EINVAL`: `sig` is invalid, the targeted signal's current disposition is
///   immutable, or `action` tries to install a handler for `SIGKILL` or
///   `SIGSTOP`.
///
/// # Safety
/// - `action`, when non-null, must be valid to read one [`OldSigaction`] for
///   the duration of the syscall.
/// - `oldaction`, when non-null, must be valid to write one [`OldSigaction`]
///   for the duration of the syscall.
/// - `oldaction`, when non-null, must not alias live Rust references or other
///   memory that would violate Rust's aliasing rules while the kernel may
///   write through that pointer.
/// - If `action` is non-null and installs a user handler or restorer address,
///   that callback state must remain valid for future signal delivery and use
///   the correct ABI expected by this historical interface.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigaction.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v7.0#n4702)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4701)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n177)
///
/// # Historical References
/// - Linux 1.0 ABI definition:
///   [include/linux/signal.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/signal.h?h=1.0#n85)
pub unsafe fn sigaction(
    sig: Int,
    action: *const OldSigaction,
    oldaction: *mut OldSigaction,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::Sigaction,
            sig as isize,
            action.addr() as isize,
            oldaction.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::offset_of;
    use std::mem::{align_of, size_of};

    use celer_system_linux_ctypes::{Int, OldSigaction};

    use crate::sys::test_support::process_global_state_guard;

    use super::sigaction;

    const SIGKILL: Int = 9;
    const SIGUSR1: Int = 10;
    const SIG_DFL: usize = 0;
    const SIG_IGN: usize = 1;

    struct RestoreSigaction {
        sig: Int,
        old: OldSigaction,
    }

    impl Drop for RestoreSigaction {
        fn drop(&mut self) {
            // SAFETY: `self.old` remains valid for the duration of the call,
            // and the null old-action pointer intentionally suppresses output.
            let _ = unsafe {
                sigaction(self.sig, &raw const self.old, core::ptr::null_mut())
            };
        }
    }

    #[test]
    fn test_old_sigaction_layout() {
        assert_eq!(size_of::<OldSigaction>(), 16);
        assert_eq!(align_of::<OldSigaction>(), 4);
        assert_eq!(offset_of!(OldSigaction, sa_handler), 0);
        assert_eq!(offset_of!(OldSigaction, sa_mask), 4);
        assert_eq!(offset_of!(OldSigaction, sa_flags), 8);
        assert_eq!(offset_of!(OldSigaction, sa_restorer), 12);
    }

    #[test]
    fn test_sigaction_query_and_restore() {
        let _guard = process_global_state_guard();

        let mut original = OldSigaction {
            sa_handler: 0,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `original` is writable for one `OldSigaction`, and the null
        // action pointer requests a query-only call.
        let rc =
            unsafe { sigaction(SIGUSR1, core::ptr::null(), &raw mut original) };
        assert_eq!(rc, 0, "query sigaction failed: {rc}");

        let _restore = RestoreSigaction {
            sig: SIGUSR1,
            old: original,
        };

        let ignore = OldSigaction {
            sa_handler: SIG_IGN,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `ignore` is readable for one `OldSigaction`, and the null
        // old-action pointer intentionally skips returning the previous value.
        let rc = unsafe {
            sigaction(SIGUSR1, &raw const ignore, core::ptr::null_mut())
        };
        assert_eq!(rc, 0, "installing SIG_IGN failed: {rc}");

        let mut current = OldSigaction {
            sa_handler: 0,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `current` is writable for one `OldSigaction`, and the null
        // action pointer requests a query-only call.
        let rc =
            unsafe { sigaction(SIGUSR1, core::ptr::null(), &raw mut current) };
        assert_eq!(rc, 0, "query after install failed: {rc}");
        assert_eq!(current.sa_handler, SIG_IGN);
    }

    #[test]
    fn test_sigaction_rejects_sigkill_handler_install() {
        let ignore = OldSigaction {
            sa_handler: SIG_IGN,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };

        // SAFETY: `ignore` is readable for one `OldSigaction`, and the null
        // old-action pointer intentionally skips returning the previous value.
        let rc = unsafe {
            sigaction(SIGKILL, &raw const ignore, core::ptr::null_mut())
        };

        assert_eq!(rc, -22, "expected EINVAL for SIGKILL, got {rc}");
    }

    #[test]
    fn test_sigaction_rejects_invalid_signal() {
        // SAFETY: both null pointers are explicitly accepted by the ABI.
        let rc =
            unsafe { sigaction(0, core::ptr::null(), core::ptr::null_mut()) };

        assert_eq!(rc, -22, "expected EINVAL for signal 0, got {rc}");
    }

    #[test]
    fn test_sigaction_faults_on_invalid_oldaction_pointer() {
        // SAFETY: this intentionally passes an invalid kernel ABI pointer to
        // verify the syscall reports `EFAULT`.
        let bad_oldaction =
            core::ptr::without_provenance_mut::<OldSigaction>(usize::MAX);
        let rc =
            unsafe { sigaction(SIGUSR1, core::ptr::null(), bad_oldaction) };

        assert_eq!(rc, -14, "expected EFAULT for bad oldaction, got {rc}");
    }

    #[test]
    fn test_sigaction_installs_before_oldaction_efault() {
        let _guard = process_global_state_guard();

        let mut original = OldSigaction {
            sa_handler: 0,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `original` is writable for one `OldSigaction`, and the null
        // action pointer requests a query-only call.
        let rc =
            unsafe { sigaction(SIGUSR1, core::ptr::null(), &raw mut original) };
        assert_eq!(rc, 0, "query sigaction failed: {rc}");

        let _restore = RestoreSigaction {
            sig: SIGUSR1,
            old: original,
        };

        let baseline = OldSigaction {
            sa_handler: SIG_IGN,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `baseline` is readable for one `OldSigaction`, and the null
        // old-action pointer intentionally skips returning the previous value.
        let rc = unsafe {
            sigaction(SIGUSR1, &raw const baseline, core::ptr::null_mut())
        };
        assert_eq!(rc, 0, "installing baseline SIG_IGN failed: {rc}");

        let new_handler = SIG_DFL;
        let new_action = OldSigaction {
            sa_handler: new_handler,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };

        // SAFETY: `new_action` is readable for one `OldSigaction`, and the
        // invalid output pointer intentionally exercises the kernel's
        // install-then-copyout-fails path on current kernels.
        let bad_oldaction =
            core::ptr::without_provenance_mut::<OldSigaction>(usize::MAX);
        let rc =
            unsafe { sigaction(SIGUSR1, &raw const new_action, bad_oldaction) };
        assert_eq!(rc, -14, "expected EFAULT for bad oldaction, got {rc}");

        let mut current = OldSigaction {
            sa_handler: 0,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };
        // SAFETY: `current` is writable for one `OldSigaction`, and the null
        // action pointer requests a query-only call.
        let rc =
            unsafe { sigaction(SIGUSR1, core::ptr::null(), &raw mut current) };
        assert_eq!(rc, 0, "query after EFAULT failed: {rc}");
        assert_eq!(current.sa_handler, new_handler);
    }
}
