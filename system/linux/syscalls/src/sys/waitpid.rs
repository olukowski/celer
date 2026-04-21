use celer_system_linux_ctypes::{Int, PidT};

use crate::arch::current::{Sysno, syscall3};

/// Wait for a child process to change state.
///
/// This is the compatibility `waitpid` entry point on x86 Linux.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: x86 Linux only for now
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Waits for the child identified by `pid` to change state.
/// - Stores the child status word in `stat_addr` when that pointer is not null.
/// - Returns the PID of the child that changed state, or a negative errno value on failure.
///
/// # Errors
/// - `ENOSYS`: The system call is not supported on this architecture.
/// - `ECHILD`: There are no unwaited-for children matching the request.
/// - `EINTR`: The wait was interrupted by a signal.
///
/// # Safety
/// - `stat_addr`, when non-null, must be valid to write a single `Int` value
///   for the duration of the syscall.
/// - The caller must ensure the `pid` and `options` arguments are valid for the
///   kernel's `waitpid(2)` ABI.
/// - Any irreversible side effects of reaping a child are intended.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/waitpid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/exit.c?h=v6.19#n1917)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/exit.c?h=v6.18.18#n1917)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n22)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=0.10#n134)
pub unsafe fn waitpid(pid: PidT, stat_addr: *mut Int, options: Int) -> PidT {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(
            Sysno::Waitpid,
            pid as isize,
            stat_addr.addr() as isize,
            options as isize,
        )
    }) as PidT
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::waitpid;
    use crate::sys::{exit, fork};

    #[test]
    fn test_waitpid() {
        let pid = fork();

        #[cfg_attr(coverage_nightly, coverage(off))] // llvm-cov can't track across the `fork` boundary
        fn use_pid(pid: PidT) {
            if pid == 0 {
                exit(0);
            }
        }

        use_pid(pid);

        let mut status: Int = 0;
        let waited = unsafe { waitpid(pid, &mut status, 0) };

        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
    }
}
