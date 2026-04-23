use celer_system_linux_ctypes::{Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall1};

/// Set the calling thread's x86 I/O privilege level.
///
/// Linux 1.0 directly updated the saved user `EFLAGS` IOPL bits for the
/// calling task. Current x86 kernels keep the historical syscall entry point
/// but emulate the privilege level with per-thread state and the I/O bitmap.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 patched the saved user `EFLAGS` directly;
///   current x86 kernels emulate IOPL and privilege-gate only upward changes
/// - Availability: present on supported x86 Linux kernels; current kernels
///   built without `CONFIG_X86_IOPL_IOPERM` return `ENOSYS`
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser for any successful change.
/// - Current kernels: raising the emulated IOPL above the current thread's
///   level requires `CAP_SYS_RAWIO` and must not be blocked by kernel
///   lockdown; lowering the level does not require that capability.
///
/// # Behavior
/// - `level` must be in the inclusive range `0..=3`.
/// - Linux 1.0 stores the requested level in the saved user `EFLAGS` IOPL
///   bits when the call succeeds.
/// - Current x86 kernels return success immediately when `level` already
///   matches the calling thread's emulated IOPL.
/// - Current x86 kernels use `level == 3` to grant full I/O-port access
///   through emulation; lower levels remove that elevation.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EINVAL`: `level > 3`.
/// - `EPERM`: Linux 1.0 rejects callers that are not superuser; current
///   kernels reject upward transitions without `CAP_SYS_RAWIO` or when kernel
///   lockdown forbids I/O-port access.
/// - `ENOSYS`: current kernels were built without `CONFIG_X86_IOPL_IOPERM`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/iopl.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/ioport.c?h=v7.0#n179)
/// - LTS: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/ioport.c?h=v7.0#n179)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/ioport.c?h=1.0#n142)
pub fn iopl(level: UnsignedInt) -> Int {
    // SAFETY: `iopl` takes only an integer argument and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall1(Sysno::Iopl, level as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, UnsignedInt};

    use crate::arch::current::{Sysno, syscall1};

    use super::iopl;

    const ENOSYS: Int = -(38 as Int);
    const EINVAL: Int = -(22 as Int);
    const EPERM: Int = -(1 as Int);

    fn raw_iopl(level: UnsignedInt) -> Int {
        // SAFETY: same integer-only argument as the wrapper under test.
        unsafe { syscall1(Sysno::Iopl, level as isize) as Int }
    }

    #[test]
    fn test_iopl_sysno() {
        assert_eq!(Sysno::Iopl as isize, 110);
    }

    #[test]
    fn test_iopl_rejects_invalid_level_or_is_unavailable() {
        let wrapped = iopl(4 as UnsignedInt);
        let raw = raw_iopl(4 as UnsignedInt);

        assert_eq!(wrapped, raw, "iopl wrapper should match raw syscall");
        assert!(
            wrapped == EINVAL || wrapped == ENOSYS,
            "expected EINVAL or ENOSYS from iopl(4), got {wrapped}",
        );
    }

    #[test]
    fn test_iopl_raise_requires_privilege_or_is_unavailable() {
        let wrapped = iopl(3 as UnsignedInt);
        let raw = raw_iopl(3 as UnsignedInt);

        assert_eq!(wrapped, raw, "iopl wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == EPERM || wrapped == ENOSYS,
            "expected success, EPERM, or ENOSYS from iopl(3), got {wrapped}",
        );

        if wrapped == 0 {
            let lower = iopl(0 as UnsignedInt);
            assert_eq!(lower, 0, "iopl(0) should drop a raised level");
        }
    }

    #[test]
    fn test_iopl_same_level_noop_succeeds_when_raised() {
        let raised = iopl(3 as UnsignedInt);

        if raised == 0 {
            let same_level = iopl(3 as UnsignedInt);
            assert_eq!(same_level, 0, "repeating iopl(3) should be a no-op");

            let lower = iopl(0 as UnsignedInt);
            assert_eq!(lower, 0, "iopl(0) should drop a raised level");
        } else {
            assert!(
                raised == EPERM || raised == ENOSYS,
                "expected EPERM or ENOSYS when iopl(3) cannot be raised, got {raised}",
            );
        }
    }

    #[test]
    fn test_iopl_zero_level_succeeds_or_is_unavailable() {
        let wrapped = iopl(0 as UnsignedInt);
        let raw = raw_iopl(0 as UnsignedInt);

        assert_eq!(wrapped, raw, "iopl wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == ENOSYS,
            "expected success or ENOSYS from iopl(0), got {wrapped}",
        );
    }
}
