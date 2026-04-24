use celer_system_linux_ctypes::{Int, PidT};

use crate::arch::current::{Sysno, syscall3};

/// Wait for a child process to change state.
///
/// This is the compatibility `waitpid` entry point on x86 Linux. On both
/// Linux 1.0 and current x86 kernels, it is implemented as `wait4(..., NULL)`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 accepted unknown option bits without
///   rejecting them; current kernels reject unsupported option bits with
///   `EINVAL` and reject `pid == INT_MIN` with `ESRCH`.
/// - Availability: x86 Linux only for now
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Waits for the child identified by `pid` to change state.
/// - Stores the child status word in `stat_addr` when that pointer is not null.
/// - Returns the PID of the child that changed state, or a negative errno
///   value on failure.
///
/// # Errors
/// - `ECHILD`: There are no unwaited-for children matching the request.
/// - `EINTR`: The wait was interrupted by a signal.
/// - `EFAULT`: `stat_addr` is not writable for one wait-status word.
/// - `EINVAL`: on current kernels, `options` contains unsupported bits.
/// - `ESRCH`: on current kernels, `pid == INT_MIN`.
///
/// # Safety
/// - `stat_addr`, when non-null, must be valid to write a single `Int` value
///   for the duration of the syscall.
/// - `stat_addr`, when non-null, must not alias live Rust references or other
///   memory that would violate Rust's aliasing rules while the kernel may
///   write through that pointer.
/// - The caller must ensure the `pid` and `options` arguments are valid for the
///   kernel's `waitpid(2)` ABI.
/// - Any irreversible side effects of reaping a child are intended.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/waitpid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/exit.c?h=v6.19#n1918)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/exit.c?h=v6.18.18#n1913)
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PidT};

    use super::waitpid;
    use crate::sys::{exit, fork};

    const WNOHANG: Int = 1;

    #[test]
    fn test_waitpid() {
        let pid = fork();
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

    #[test]
    fn test_waitpid_returns_echild_without_children() {
        let rc = unsafe { waitpid(-1, core::ptr::null_mut(), 0) };

        assert_eq!(rc, -10, "expected ECHILD, got {rc}");
    }

    #[test]
    fn test_waitpid_rejects_invalid_options() {
        let rc = unsafe { waitpid(-1, core::ptr::null_mut(), -1) };

        assert_eq!(rc, -22, "expected EINVAL, got {rc}");
    }

    #[test]
    fn test_waitpid_reports_efault_for_bad_status_pointer() {
        let pid = fork();
        if pid == 0 {
            exit(0);
        }

        let bad_status = core::ptr::without_provenance_mut::<Int>(usize::MAX);
        let rc = unsafe { waitpid(pid, bad_status, 0) };

        assert_eq!(rc, -14, "expected EFAULT, got {rc}");

        let _ = unsafe { waitpid(pid, core::ptr::null_mut(), WNOHANG) };
    }

    #[test]
    fn test_waitpid_int_min_pid_returns_esrch() {
        let rc = unsafe { waitpid(Int::MIN, core::ptr::null_mut(), 0) };

        assert_eq!(rc, -3, "expected ESRCH for pid == INT_MIN, got {rc}");
    }
}
