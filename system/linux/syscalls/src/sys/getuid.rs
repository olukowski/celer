use celer_system_linux_ctypes::OldUidT;

use crate::arch::current::{Sysno, syscall0};

/// Return the real user ID through the legacy i386 `getuid16` ABI.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 still uses the legacy 16-bit `getuid`
///   entry (`sys_getuid16`)
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Returns the caller's real user ID mapped into the caller's user namespace
///   through `from_kuid_munged(current_user_ns(), current_uid())`.
/// - Current kernels then convert that namespace ID with `high2lowuid`: values
///   above 65535 are returned as `overflowuid`, so the legacy 16-bit result is
///   lossy for high user IDs.
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getuid.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n203)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n203)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n738)
/// - Current high-ID conversion:
///   [include/linux/highuid.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/linux/highuid.h?h=v7.0#n47)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n362)
pub fn getuid16() -> OldUidT {
    // SAFETY: `getuid16` takes no arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall0(Sysno::Getuid) as OldUidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::getuid16;
    use celer_system_linux_ctypes::OldUidT;

    #[test]
    fn test_getuid() {
        let uid = getuid16();
        let expected = unsafe { syscall0(Sysno::Getuid) as OldUidT };

        assert_eq!(uid, expected, "getuid16 should match the raw syscall");
    }
}
