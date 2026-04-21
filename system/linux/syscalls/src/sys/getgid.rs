use celer_system_linux_ctypes::GidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the real group ID (GID) of the calling process.
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
/// - `man` [page](https://man7.org/linux/man-pages/man2/getgid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1039)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1039)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n748)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n372)
pub fn getgid() -> GidT {
    // SAFETY: `getgid` is always safe to call.
    (unsafe { syscall0(Sysno::Getgid) }) as GidT
}

#[cfg(test)]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::getgid;
    use celer_system_linux_ctypes::GidT;

    #[test]
    fn test_getgid() {
        let gid = getgid();
        let expected = (unsafe { syscall0(Sysno::Getgid) }) as GidT;

        assert_eq!(gid, expected, "getgid should match the raw syscall");
    }
}
