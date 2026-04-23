use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall3};

/// Set the nice value for processes selected by `which` and `who`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 converts `niceval` to an internal scheduler
///   priority with `PZERO - niceval` and floors the result at `1`; current
///   kernels clamp `niceval` to the standard nice range `[-20, 19]`, use
///   namespace-aware permission checks, and consult `security_task_setnice()`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None when targeting only tasks owned by the caller and not requesting a
///   nice reduction that requires extra privilege.
/// - Linux 1.0 required the superuser to raise scheduler priority or target
///   tasks not owned by the caller.
/// - Current kernels use `CAP_SYS_NICE` and may also reject the operation via
///   LSM policy.
///
/// # Behavior
/// - `which` accepts `0` (`PRIO_PROCESS`), `1` (`PRIO_PGRP`), or `2`
///   (`PRIO_USER`).
/// - `who == 0` means the calling task, the caller's process group, or the
///   caller's UID depending on `which`.
/// - For `PRIO_PGRP` and `PRIO_USER`, both Linux 1.0 and current kernels may
///   update some matching tasks before returning an error from a later match.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EINVAL`: `which` is outside the supported selector range.
/// - `ESRCH`: no task matched the selected `which` / `who` target.
/// - `EPERM`: the caller lacks permission to change a matched task owned by
///   another user.
/// - `EACCES`: the caller tried to reduce nice value without sufficient
///   privilege.
/// - Current kernels may also return an LSM-specific negative errno from
///   `security_task_setnice()`.
/// - For `PRIO_PGRP` and `PRIO_USER`, an error return does not guarantee that
///   no matching task was updated earlier in the same syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setpriority.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n259)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n259)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n52)
pub fn setpriority(which: Int, who: Int, niceval: Int) -> Int {
    // SAFETY: `setpriority` takes only scalar arguments and has no caller-side
    // memory-safety precondition.
    unsafe {
        syscall3(
            Sysno::Setpriority,
            which as isize,
            who as isize,
            niceval as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::setpriority;

    const PRIO_PROCESS: Int = 0;

    #[test]
    fn test_setpriority_current_process_nice_19() {
        let rc = setpriority(PRIO_PROCESS, 0, 19);

        assert_eq!(rc, 0, "setpriority(PRIO_PROCESS, 0, 19) failed: {rc}");
    }

    #[test]
    fn test_setpriority_invalid_selector() {
        let rc = setpriority(3, 0, 0);

        assert_eq!(rc, -22, "setpriority(3, 0, 0) returned {rc}");
    }

    #[test]
    fn test_setpriority_missing_process() {
        let rc = setpriority(PRIO_PROCESS, -1, 0);

        assert_eq!(rc, -3, "setpriority(PRIO_PROCESS, -1, 0) returned {rc}",);
    }
}
