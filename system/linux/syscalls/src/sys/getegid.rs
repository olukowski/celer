use celer_system_linux_ctypes::OldGidT;

use crate::arch::current::{Sysno, syscall0};

/// Return the effective group ID through the legacy i386 `getegid16` ABI.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 still uses the legacy 16-bit `getegid`
///   entry (`sys_getegid16`)
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Returns the caller's effective group ID mapped into the caller's user
///   namespace.
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getegid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1045)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1045)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n753)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n377)
pub fn getegid16() -> OldGidT {
    // SAFETY: `getegid16` takes no arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall0(Sysno::Getegid) as OldGidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::getegid16;
    use celer_system_linux_ctypes::OldGidT;

    #[test]
    fn test_getegid() {
        let egid = getegid16();
        let expected = unsafe { syscall0(Sysno::Getegid) as OldGidT };

        assert_eq!(egid, expected, "getegid16 should match the raw syscall");
    }
}
