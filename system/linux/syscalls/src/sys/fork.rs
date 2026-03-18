use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Create a new process by duplicating the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Once this system call returns, both the parent and child processes continue execution
///   right after the `fork` call.
/// - In the child process, `fork` returns 0.
/// - In the parent process, `fork` returns the PID of the child process.
///
/// # Errors
/// - `ENOSYS`: The system call is not supported on this architecture.
/// - `EINVAL`: The system call is not usable (e.g. kernel configued without `CONFIG_MMU`).
/// - `EAGAIN`: The system is out of process resources.
/// - `EAGAIN`: The caller is using the `SCHED_DEADLINE` policy and does not have the `reset-on-fork` flag set.
/// - `ENOMEM`: The system is low on memory and failed to allocate the necessary resources for the child process.
/// - `ENOMEM`: The "init" process of this PID namespace has terminated.
/// - `ERESTARTNOINTR`: The system call was interrupted by a signal and will be automatically restarted (only visible in a trace).
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fork.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/fork.c?h=v6.19#n2731)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/fork.c?h=v6.18.18#n2689)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/fork.c?h=1.0#n124)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/system_call.s?h=0.10#n162)
pub fn fork() -> PidT {
    // SAFETY: `fork` is safe to call.
    (unsafe { syscall0(Sysno::Fork) }) as PidT
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use super::fork;

    #[test]
    fn test_fork() {
        let pid = fork();

        #[cfg_attr(coverage_nightly, coverage(off))] // llvm-cov can't track across the `fork` boundary
        fn use_pid(pid: PidT) {
            if pid == 0 {
                // child
            } else {
                // parent, though we might have an error here if `pid < 0`
            }
        }

        use_pid(pid);
    }
}
