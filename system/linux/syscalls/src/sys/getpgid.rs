use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall1};

/// Returns the process group ID (PGID) of the selected process.
///
/// This wrapper exposes the raw kernel return value. Successful calls return a
/// non-negative process group ID; negative values encode errno.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 returns the raw `task_struct.pgrp` field,
///   while current kernels return the namespace-visible PGID after an LSM hook
///   check.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `pid == 0` returns the calling process's current process group ID.
/// - Otherwise, the kernel looks up the supplied process ID and returns that
///   task's process group ID.
///
/// # Errors
/// - `ESRCH`: no task matches `pid`, or current kernels find a task but no
///   process group to report for it.
///
/// Current kernels may also propagate an LSM-defined negative errno from
/// `security_task_getpgid()`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getpgid.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n1215)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1215)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n509)
///
/// # Historical References
/// - Linux 1.0 syscall number table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n141)
pub fn getpgid(pid: PidT) -> PidT {
    // SAFETY: `getpgid` takes only an integer PID selector and has no
    // caller-visible memory-safety precondition.
    unsafe { syscall1(Sysno::Getpgid, pid as isize) as PidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use crate::{
        arch::current::{Sysno, syscall1},
        sys::{getpid, test_support::getpgrp},
    };

    use super::getpgid;

    #[test]
    fn test_getpgid_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Getpgid as isize, 132);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Getpgid as isize, 155);
    }

    #[test]
    fn test_getpgid_zero_matches_getpgrp() {
        let pgid = getpgid(0);

        assert_eq!(
            pgid,
            unsafe { getpgrp() },
            "getpgid(0) should match the caller's process group"
        );
    }

    #[test]
    fn test_getpgid_self_matches_getpgrp() {
        let pgid = getpgid(getpid());

        assert_eq!(
            pgid,
            unsafe { getpgrp() },
            "getpgid(getpid()) should match the caller's process group"
        );
    }

    #[test]
    fn test_getpgid_nonexistent_process_returns_esrch() {
        let pgid = getpgid(PidT::MAX);

        assert_eq!(pgid, -3, "expected ESRCH for nonexistent PID, got {pgid}");
    }

    #[test]
    fn test_getpgid_matches_raw_syscall() {
        let wrapped = getpgid(getpid());
        // SAFETY: same integer-only argument as the wrapper.
        let raw =
            unsafe { syscall1(Sysno::Getpgid, getpid() as isize) as PidT };

        assert_eq!(wrapped, raw, "getpgid should match the raw syscall");
    }
}
