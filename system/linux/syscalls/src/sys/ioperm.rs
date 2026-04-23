use celer_system_linux_ctypes::{Int, UnsignedLong};

use crate::arch::current::{Sysno, syscall3};

/// Enable or disable access to a range of x86 I/O ports for the calling
/// thread.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 required the superuser for both enabling and
///   disabling permissions and limited the bitmap to 1024 ports; current x86
///   kernels require `CAP_SYS_RAWIO` only when enabling permissions, allow
///   unprivileged revocation, and extend the bitmap to 65536 ports
/// - Availability: present on supported x86 Linux kernels; current kernels
///   built without `CONFIG_X86_IOPL_IOPERM` return `ENOSYS`
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser for any change.
/// - Current kernels: enabling access requires `CAP_SYS_RAWIO` and must not
///   be blocked by kernel lockdown; disabling access does not require that
///   capability.
///
/// # Behavior
/// - `from` names the first I/O port in the range.
/// - `num` names the number of consecutive ports in the range.
/// - `turn_on == true` grants access to the selected range.
/// - `turn_on == false` revokes access to the selected range.
/// - Linux 1.0 rejects ranges outside its 1024-port bitmap.
/// - Current kernels reject ranges outside their 65536-port bitmap.
/// - Current kernels lazily allocate the per-thread bitmap the first time a
///   permission is enabled.
/// - Current kernels return success for a revoke request even when the thread
///   does not yet have an allocated bitmap.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EINVAL`: `from + num` overflows or exceeds the kernel's I/O permission
///   bitmap range.
/// - `EPERM`: Linux 1.0 rejects callers that are not superuser; current
///   kernels reject enable requests from callers without `CAP_SYS_RAWIO` or
///   when kernel lockdown forbids I/O port access.
/// - `ENOMEM`: current kernels cannot allocate or duplicate the per-thread
///   I/O permission bitmap.
/// - `ENOSYS`: current kernels were built without `CONFIG_X86_IOPL_IOPERM`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/ioperm.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/ioport.c?h=v7.0#n71)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/ioport.c?h=v6.18.18#n71)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/ioport.c?h=1.0#n114)
pub fn ioperm(from: UnsignedLong, num: UnsignedLong, turn_on: bool) -> Int {
    let turn_on = if turn_on { 1 } else { 0 };

    // SAFETY: `ioperm` takes only integer arguments and has no
    // caller-visible memory-safety preconditions.
    (unsafe {
        syscall3(Sysno::Ioperm, from as isize, num as isize, turn_on as isize)
    }) as Int
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, UnsignedLong};

    use crate::arch::current::{Sysno, syscall3};

    use super::ioperm;

    const ENOSYS: Int = -(38 as Int);
    const EINVAL: Int = -(22 as Int);
    const EPERM: Int = -(1 as Int);

    fn raw_ioperm(from: UnsignedLong, num: UnsignedLong, turn_on: bool) -> Int {
        let turn_on = if turn_on { 1 } else { 0 };

        // SAFETY: same integer-only arguments as the wrapper under test.
        unsafe {
            syscall3(
                Sysno::Ioperm,
                from as isize,
                num as isize,
                turn_on as isize,
            ) as Int
        }
    }

    #[test]
    fn test_ioperm_sysno() {
        assert_eq!(Sysno::Ioperm as isize, 101);
    }

    #[test]
    fn test_ioperm_zero_length_range_is_rejected_or_unavailable() {
        let wrapped = ioperm(0 as UnsignedLong, 0 as UnsignedLong, true);
        let raw = raw_ioperm(0 as UnsignedLong, 0 as UnsignedLong, true);

        assert_eq!(wrapped, raw, "ioperm wrapper should match raw syscall");
        assert!(
            wrapped == EINVAL || wrapped == ENOSYS,
            "expected EINVAL or ENOSYS from ioperm(0, 0, true), got {wrapped}",
        );
    }

    #[test]
    fn test_ioperm_enable_matches_raw_syscall() {
        let port = 0x80 as UnsignedLong;
        let wrapped = ioperm(port, 1 as UnsignedLong, true);
        let raw = raw_ioperm(port, 1 as UnsignedLong, true);

        assert_eq!(wrapped, raw, "ioperm wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == EPERM || wrapped == ENOSYS,
            "expected success, EPERM, or ENOSYS from ioperm enable, got {wrapped}",
        );

        if wrapped == 0 {
            let revoke = ioperm(port, 1 as UnsignedLong, false);
            assert_eq!(
                revoke, 0,
                "ioperm revoke should clean up granted access"
            );
        }
    }

    #[test]
    fn test_ioperm_out_of_bounds_range_is_rejected_or_unavailable() {
        let wrapped = ioperm(65_535 as UnsignedLong, 2 as UnsignedLong, false);
        let raw = raw_ioperm(65_535 as UnsignedLong, 2 as UnsignedLong, false);

        assert_eq!(wrapped, raw, "ioperm wrapper should match raw syscall");
        assert!(
            wrapped == EINVAL || wrapped == ENOSYS,
            "expected EINVAL or ENOSYS from out-of-bounds ioperm, got {wrapped}",
        );
    }

    #[test]
    fn test_ioperm_revoke_without_bitmap_succeeds_or_is_unavailable() {
        let port = 0x80 as UnsignedLong;
        let wrapped = ioperm(port, 1 as UnsignedLong, false);
        let raw = raw_ioperm(port, 1 as UnsignedLong, false);

        assert_eq!(wrapped, raw, "ioperm wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == ENOSYS,
            "expected success or ENOSYS from ioperm revoke, got {wrapped}",
        );
    }
}
