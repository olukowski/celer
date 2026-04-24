use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Create a new process by duplicating the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes:
///   - Linux 1.0 implemented `fork` directly in `sys_fork`.
///   - Current kernels route `fork` through `kernel_clone()` with
///     `exit_signal = SIGCHLD`.
/// - Availability: present on supported MMU x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Once this system call returns, both the parent and child continue
///   execution immediately after the `fork` call.
/// - In the child process, `fork` returns 0.
/// - In the parent process, `fork` returns the PID of the child process.
///
/// # Errors
/// - `EAGAIN`: Linux 1.0 returns this when it cannot allocate a task slot or
///   kernel pages for the child; current kernels also use it when process or
///   thread limits block `copy_process()`.
/// - `ENOMEM`: current kernels failed to allocate child task resources.
/// - `EINVAL`: current NOMMU kernels reject `fork`.
///
/// # References
/// - Stable entry:
///   [v7.0 fork](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/fork.c?h=v7.0#n2733)
/// - Stable helper:
///   [v7.0 copy_process](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/fork.c?h=v7.0#n1964)
/// - LTS entry:
///   [v6.18.18 fork](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/fork.c?h=v6.18.18#n2689)
/// - LTS helper:
///   [v6.18.18 copy_process](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/fork.c?h=v6.18.18#n1926)
/// - First stable:
///   [Linux 1.0 sys_fork](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/fork.c?h=1.0#n124)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/system_call.s?h=0.10#n162)
pub fn fork() -> PidT {
    // SAFETY: `fork` is safe to call.
    unsafe { syscall0(Sysno::Fork) as PidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use super::fork;
    use crate::sys::{exit, waitpid};

    #[test]
    fn test_fork() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        fn handle_pid(pid: PidT) {
            if pid == 0 {
                exit(0);
            }
        }

        handle_pid(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }
}
