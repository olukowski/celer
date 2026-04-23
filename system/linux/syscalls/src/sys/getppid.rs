use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the parent process ID (PPID) of the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: current kernels read the parent through `real_parent`
///   under RCU and may return `0` when the parent is not visible in the
///   caller's active PID namespace.
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getppid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1016)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1016)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n733)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n356)
pub fn getppid() -> PidT {
    // SAFETY: `getppid` is always safe to call.
    syscall0(Sysno::Getppid) as PidT
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use crate::arch::current::{Sysno, syscall0};

    use super::getppid;

    #[test]
    fn test_getppid() {
        let ppid = getppid();
        let raw = syscall0(Sysno::Getppid) as PidT;

        assert_eq!(ppid, raw, "getppid should match the raw syscall result");
    }
}
