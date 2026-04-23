use celer_system_linux_ctypes::{Int, Long, PidT};

use crate::arch::current::{Sysno, syscall2};

/// Send signal `sig` to process `pid`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for signals the caller is permitted to send; permission checks are
///   enforced by the kernel.
///
/// # Behavior
/// - The kernel prepares siginfo with `PIDTYPE_TGID` and then forwards the
///   call to `kill_something_info()`.
/// - A signal number of `0` performs permission/existence checks without
///   delivering a signal.
///
/// # Errors
/// - The kernel may return permission, existence, and signal-validity errors
///   such as `EPERM`, `ESRCH`, and `EINVAL`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/kill.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n3947)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n3949)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=1.0#n278)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=0.10#n60)
pub fn kill(pid: PidT, sig: Int) -> Long {
    // SAFETY: `kill` takes only scalar arguments and has no Rust-side safety
    // preconditions.
    unsafe { syscall2(Sysno::Kill, pid as isize, sig as isize) as Long }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use crate::sys::getpid;

    use super::kill;

    #[test]
    fn test_kill_signal_zero() {
        let pid = getpid();

        let rc = kill(pid, 0 as Int);

        assert_eq!(rc, 0, "kill(pid, 0) failed: {rc}");
    }
}
