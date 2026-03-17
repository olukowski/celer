use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the process ID (PID) of the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
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
/// - `man` [page](https://man7.org/linux/man-pages/man2/getpid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n999)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n999)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n728)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n352)
pub fn getpid() -> PidT {
    // SAFETY: `getpid` is always safe to call.
    (unsafe { syscall0(Sysno::Getpid) }) as PidT
}

#[cfg(test)]
mod tests {
    use super::getpid;

    #[test]
    fn test_getpid() {
        let pid = getpid();

        assert!(pid > 0, "getpid should return a positive value");
    }
}
