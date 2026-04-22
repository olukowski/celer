use celer_system_linux_ctypes::{Long, PidT};

use crate::arch::current::{Sysno, syscall2};

/// Set the process group ID of the calling process or another process in the
/// same session.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 added the `did_exec` / `EACCES` branch, and
///   current kernels also reject non-thread-group leaders with `EINVAL` and
///   consult `security_task_setpgid()`.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `pid == 0` means the calling process.
/// - `pgid == 0` means `pid` after defaulting.
/// - Negative `pgid` is rejected.
/// - This is the legacy x86 32-bit `setpgid` entrypoint; it is distinct from
///   other process-group and session APIs.
///
/// # Errors
/// - `EINVAL`: `pgid` is negative, or the target task is not a thread-group
///   leader.
/// - `ESRCH`: `pid` does not identify a task that the syscall can operate on.
/// - `EPERM`: session or leadership checks fail.
/// - `EACCES`: the target has already executed a new program image in the
///   valid parent/session branch.
///
/// An LSM hook may also reject the operation with its own negative errno.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setpgid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1114)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1114)
/// - x86 table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n72)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n468)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n181)
pub fn setpgid(pid: PidT, pgid: PidT) -> Long {
    // SAFETY: the syscall only takes integer arguments and has no caller-side
    // memory-safety precondition.
    (unsafe { syscall2(Sysno::Setpgid, pid as isize, pgid as isize) }) as Long
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use super::setpgid;

    #[test]
    fn test_setpgid_negative_pgid() {
        let ret = setpgid(0 as PidT, -1 as PidT);
        assert_eq!(ret, -(22 as _));
    }
}
