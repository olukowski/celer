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
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getgid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1039)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1039)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n748)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n372)
pub fn getgid16() -> OldGidT {
    syscall0(Sysno::Getgid) as OldGidT
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
        let expected = syscall0(Sysno::Getgid) as OldGidT;

        assert_eq!(gid, expected, "getgid16 should match the raw syscall");
    }
}
