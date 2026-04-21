use celer_system_linux_ctypes::UidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the effective user ID (EUID) of the calling process.
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
/// - `man` [page](https://man7.org/linux/man-pages/man2/geteuid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1033)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1033)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n743)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n367)
pub fn geteuid() -> UidT {
    // SAFETY: `geteuid` is always safe to call.
    (unsafe { syscall0(Sysno::Geteuid) }) as UidT
}

#[cfg(test)]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::geteuid;
    use celer_system_linux_ctypes::UidT;

    #[test]
    fn test_geteuid() {
        let euid = geteuid();
        let expected = (unsafe { syscall0(Sysno::Geteuid) }) as UidT;

        assert_eq!(euid, expected, "geteuid should match the raw syscall");
    }
}
