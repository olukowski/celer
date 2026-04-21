use celer_system_linux_ctypes::UidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the real user ID (UID) of the calling process.
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
/// - `man` [page](https://man7.org/linux/man-pages/man2/getuid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1027)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1027)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n738)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n362)
pub fn getuid() -> UidT {
    // SAFETY: `getuid` is always safe to call.
    unsafe { syscall0(Sysno::Getuid) } as UidT
}

#[cfg(test)]
mod tests {
    use super::getuid;

    #[test]
    fn test_getuid() {
        let uid = getuid();

        assert!(uid >= 0, "getuid should return a non-negative value");
    }
}
