use celer_system_linux_ctypes::Int;

use crate::sys;

/// Terminate the calling thread with `status`.
///
/// This safe wrapper keeps the raw integer exit-status argument and exposes the
/// syscall's non-returning behavior as `!`.
///
/// On normal syscall dispatch this function never returns. If a syscall-entry
/// filter such as seccomp or ptrace synthesizes a return before the kernel
/// reaches `exit`, this wrapper panics so its Rust signature remains
/// non-returning.
///
/// See [`sys::exit`] for kernel behavior, reachable pre-dispatch synthetic
/// returns, and source references.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn exit(status: Int) -> ! {
    let ret = sys::exit(status);
    panic!("sys::exit returned unexpectedly with {ret}");
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::exit;
    use crate::sys::test_support::{fork, waitpid};

    #[test]
    fn test_exit_terminates_child() {
        // SAFETY: `fork` isolates the terminating wrapper call in the child.
        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {pid}");

        if pid == 0 {
            exit(0);
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
