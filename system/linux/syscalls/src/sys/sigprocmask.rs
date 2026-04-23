use celer_system_linux_ctypes::{Int, OldSigsetT};

use crate::arch::current::{Sysno, syscall3};

/// Add the selected signals to the caller's blocked mask.
pub const SIG_BLOCK: Int = 0;

/// Remove the selected signals from the caller's blocked mask.
pub const SIG_UNBLOCK: Int = 1;

/// Replace the caller's blocked mask with the selected signals.
pub const SIG_SETMASK: Int = 2;

/// Examine or change the caller's legacy blocked-signal mask word.
///
/// This is the historical i386 `sigprocmask` syscall ABI, which moves a
/// single 32-bit signal-mask word rather than the `rt_sigprocmask` ABI's
/// extensible `sigset_t`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 and current kernels both ignore `how` when
///   `set == NULL`, strip `SIGKILL` and `SIGSTOP` from any requested mask
///   update, and may still return `EFAULT` after installing a new mask if
///   copying `oset` back to user memory fails
/// - Availability: current kernels keep this obsolete entry point only on
///   arch ABIs that opt into `__ARCH_WANT_SYS_SIGPROCMASK`
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `set == NULL` performs no mask update and ignores `how`.
/// - `oset == NULL` suppresses returning the previous blocked mask.
/// - On success, returns `0`.
/// - `SIG_BLOCK`, `SIG_UNBLOCK`, and `SIG_SETMASK` are `0`, `1`, and `2`.
/// - Attempts to block `SIGKILL` or `SIGSTOP` are silently stripped from the
///   requested mask before the kernel updates the task's blocked set.
/// - `oset`, when non-null, receives the blocked mask from before any update.
///
/// # Errors
/// - `EFAULT`: `set` is outside readable user memory, or `oset` is outside
///   writable user memory.
/// - `EINVAL`: `how` is not one of [`SIG_BLOCK`], [`SIG_UNBLOCK`], or
///   [`SIG_SETMASK`] when `set` is non-null.
///
/// # Safety
/// - `set`, when non-null, must be valid to read one [`OldSigsetT`] for the
///   duration of the syscall.
/// - `oset`, when non-null, must be valid to write one [`OldSigsetT`] for the
///   duration of the syscall.
/// - `oset`, when non-null, must not alias live Rust references or other
///   memory that would violate Rust's aliasing rules while the kernel may
///   write through that pointer.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigprocmask.2.html)
/// - Current mainline:
///   [v7.0-rc7](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v7.0-rc7#n4571)
/// - Current compat note:
///   [v7.0-rc7](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/compat.c?h=v7.0-rc7#n27)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n50)
///
/// # Historical References
/// - Linux 1.0 signal constants:
///   [include/linux/signal.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/signal.h?h=1.0#n4)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n135)
pub unsafe fn sigprocmask(
    how: Int,
    set: *const OldSigsetT,
    oset: *mut OldSigsetT,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::Sigprocmask,
            how as isize,
            set.addr() as isize,
            oset.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, OldSigsetT};

    use crate::arch::current::{Sysno, syscall3};
    use crate::sys::test_support::process_global_state_guard;
    use crate::sys::{sgetmask, ssetmask};

    use super::{SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, sigprocmask};

    const EFAULT: Int = 14;
    const EINVAL: Int = 22;
    const SIGKILL: OldSigsetT = 9;
    const SIGUSR1: OldSigsetT = 10;
    const SIGUSR2: OldSigsetT = 12;
    const SIGSTOP: OldSigsetT = 19;

    struct RestoreMask(OldSigsetT);

    impl Drop for RestoreMask {
        fn drop(&mut self) {
            let _ = ssetmask(self.0);
        }
    }

    fn signal_bit(sig: OldSigsetT) -> OldSigsetT {
        (1 as OldSigsetT) << (sig - 1)
    }

    #[test]
    fn test_sigprocmask_matches_raw_syscall() {
        let mut old = 0 as OldSigsetT;
        let wrapped =
            unsafe { sigprocmask(12345, core::ptr::null(), &raw mut old) };
        assert_eq!(
            wrapped, 0,
            "sigprocmask should ignore how when set is null"
        );

        let mut raw_old = 0 as OldSigsetT;
        let raw = unsafe {
            syscall3(
                Sysno::Sigprocmask,
                12345,
                core::ptr::null::<OldSigsetT>().addr() as isize,
                (&raw mut raw_old).addr() as isize,
            )
        } as Int;
        assert_eq!(raw, 0, "raw sigprocmask should succeed: {raw}");
        assert_eq!(
            old, raw_old,
            "sigprocmask should match the raw syscall output"
        );
    }

    #[test]
    fn test_sigprocmask_updates_and_reports_previous_mask() {
        let _guard = process_global_state_guard();

        let original = ssetmask(0);
        let _restore = RestoreMask(original);

        let requested =
            signal_bit(SIGUSR1) | signal_bit(SIGKILL) | signal_bit(SIGSTOP);
        let mut old = 0 as OldSigsetT;
        let rc = unsafe {
            sigprocmask(SIG_SETMASK, &raw const requested, &raw mut old)
        };
        assert_eq!(rc, 0, "sigprocmask failed: {rc}");
        assert_eq!(old, 0, "sigprocmask should report the previous mask");

        let effective = sgetmask();
        assert_eq!(
            effective,
            signal_bit(SIGUSR1),
            "sigprocmask should strip SIGKILL and SIGSTOP"
        );

        let requested = signal_bit(SIGUSR2);
        let rc = unsafe {
            sigprocmask(SIG_BLOCK, &raw const requested, core::ptr::null_mut())
        };
        assert_eq!(rc, 0, "sigprocmask(SIG_BLOCK) failed: {rc}");

        let effective = sgetmask();
        assert_eq!(
            effective,
            signal_bit(SIGUSR1) | signal_bit(SIGUSR2),
            "sigprocmask(SIG_BLOCK) should add to the blocked mask"
        );

        let requested = signal_bit(SIGUSR1);
        let rc = unsafe {
            sigprocmask(
                SIG_UNBLOCK,
                &raw const requested,
                core::ptr::null_mut(),
            )
        };
        assert_eq!(rc, 0, "sigprocmask(SIG_UNBLOCK) failed: {rc}");

        let effective = sgetmask();
        assert_eq!(
            effective,
            signal_bit(SIGUSR2),
            "sigprocmask(SIG_UNBLOCK) should remove from the blocked mask"
        );
    }

    #[test]
    fn test_sigprocmask_rejects_invalid_how_when_set_is_present() {
        let requested = signal_bit(SIGUSR1);
        let rc = unsafe {
            sigprocmask(99, &raw const requested, core::ptr::null_mut())
        };

        assert_eq!(rc, -EINVAL, "expected EINVAL for bad how, got {rc}");
    }

    #[test]
    fn test_sigprocmask_faults_on_invalid_set_pointer_before_updating() {
        let _guard = process_global_state_guard();

        let original = ssetmask(signal_bit(SIGUSR1));
        let _restore = RestoreMask(original);

        let bad_set = core::ptr::without_provenance::<OldSigsetT>(usize::MAX);
        let rc =
            unsafe { sigprocmask(SIG_SETMASK, bad_set, core::ptr::null_mut()) };
        assert_eq!(
            rc, -EFAULT,
            "expected EFAULT for bad set pointer, got {rc}"
        );

        let effective = ssetmask(0);
        assert_eq!(
            effective,
            signal_bit(SIGUSR1),
            "a bad set pointer must leave the mask unchanged"
        );
    }

    #[test]
    fn test_sigprocmask_updates_before_invalid_oset_fault() {
        let _guard = process_global_state_guard();

        let original = ssetmask(0);
        let _restore = RestoreMask(original);

        let requested = signal_bit(SIGUSR2);
        let bad_oset =
            core::ptr::without_provenance_mut::<OldSigsetT>(usize::MAX);
        let rc =
            unsafe { sigprocmask(SIG_SETMASK, &raw const requested, bad_oset) };
        assert_eq!(
            rc, -EFAULT,
            "expected EFAULT for bad oset pointer, got {rc}"
        );

        let effective = ssetmask(0);
        assert_eq!(
            effective, requested,
            "sigprocmask should install the new mask before reporting oset EFAULT"
        );
    }
}
