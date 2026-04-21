use celer_system_linux_ctypes::GidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the effective group ID (EGID) of the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present
///
/// # Required Privileges
/// - None
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
pub fn getegid() -> GidT {
    // SAFETY: `getegid` is always safe to call.
    (unsafe { syscall0(Sysno::Getegid) }) as GidT
}

#[cfg(test)]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::getegid;
    use celer_system_linux_ctypes::GidT;

    #[test]
    fn test_getegid() {
        let egid = getegid();
        let expected = (unsafe { syscall0(Sysno::Getegid) }) as GidT;

        assert_eq!(egid, expected, "getegid should match the raw syscall");
    }
}
