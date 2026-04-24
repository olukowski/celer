use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall1};

/// Terminates the calling thread.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present
///
/// # Required privileges
/// - None
///
/// # Behavior
/// - Terminates only the calling **thread**, not the entire process.
/// - If the kernel dispatches to the `exit` implementation, this syscall does
///   not return.
/// - Current kernels may skip syscall dispatch before the `exit`
///   implementation runs, for example through seccomp or ptrace syscall-entry
///   handling. If dispatch is skipped, this wrapper returns the raw
///   synthesized syscall result.
///
/// # Errors
/// - The `exit` implementation itself never returns an errno.
/// - Syscall-entry filters may synthesize a negative errno-shaped return value
///   before dispatch.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/exit.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/exit.c?h=v6.19#n1077)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/exit.c?h=v6.18.18#n1072)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=1.0#n479)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=0.10#n129)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn exit(error_code: Int) -> Int {
    // SAFETY: `exit` is a safe syscall.
    unsafe { syscall1(Sysno::Exit, error_code as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::exit;
    use crate::sys::test_support::{fork, waitpid};

    #[test]
    fn test_exit_terminates_child_when_dispatched() {
        // SAFETY: `fork` is used here to isolate the `exit` syscall under
        // test from the test process itself.
        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {pid}");

        if pid == 0 {
            let _ = exit(0);
            std::process::abort();
        }

        let mut status = 0 as Int;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };

        assert_eq!(waited, pid, "waitpid failed: {waited}");
        assert_eq!(status & 0x7f, 0, "child died from signal: {status}");
        assert_eq!(
            (status >> 8) & 0xff,
            0,
            "child exited with nonzero status: {status}"
        );
    }
}
