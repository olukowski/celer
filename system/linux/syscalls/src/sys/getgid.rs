use celer_system_linux_ctypes::OldGidT;

use crate::arch::current::{Sysno, syscall0};

/// Return the real group ID through the legacy i386 `getgid16` ABI.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 still uses the legacy 16-bit `getgid`
///   entry (`sys_getgid16`)
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Returns the caller's real group ID mapped into the caller's user namespace
///   through `from_kgid_munged(current_user_ns(), current_gid())`.
/// - Current kernels then convert that namespace ID with `high2lowgid`: values
///   above 65535 are returned as `overflowgid`, so the legacy 16-bit result is
///   lossy for high group IDs.
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getgid.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n213)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n213)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n748)
/// - Current high-ID conversion:
///   [include/linux/highuid.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/linux/highuid.h?h=v7.0#n48)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n372)
pub fn getgid16() -> OldGidT {
    // SAFETY: `getgid16` takes no arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall0(Sysno::Getgid) as OldGidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::getgid16;
    use celer_system_linux_ctypes::OldGidT;

    #[test]
    fn test_getgid() {
        let gid = getgid16();
        let expected = unsafe { syscall0(Sysno::Getgid) as OldGidT };

        assert_eq!(gid, expected, "getgid16 should match the raw syscall");
    }
}
