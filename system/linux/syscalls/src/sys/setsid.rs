use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Creates a new session and makes the caller its leader.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 tightened the legacy leader check to return
///   `EPERM` unconditionally for existing session leaders; current kernels
///   still fail with `EPERM`, but broaden that check to also reject an
///   existing process group with the proposed session ID and perform the
///   change under tasklist locking.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, Linux 1.0 sets the caller's `leader` flag.
/// - On success, Linux 1.0 sets both the new session ID and process group ID
///   to the caller's PID.
/// - On success, Linux 1.0 detaches the caller from its controlling terminal
///   by setting `tty` to `-1`.
/// - Returns the new process group ID on success.
///
/// # Errors
/// - `EPERM`: Linux 1.0 rejects the call if the caller is already a session
///   leader.
///
/// Current kernels also use `EPERM` when a process group already exists with
/// the proposed session ID.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setsid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1303)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1303)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n527)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n204)
pub fn setsid() -> PidT {
    // SAFETY: `setsid` takes no arguments and has no caller-visible
    // memory-safety precondition.
    unsafe { syscall0(Sysno::Setsid) as PidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use crate::sys::{exit, fork, getpgrp, getpid, waitpid};

    use super::setsid;

    #[test]
    fn test_setsid_local_process_smoke() {
        let ret = setsid();

        assert!(ret != 0);
        assert!(
            ret >= -1,
            "setsid should either succeed or report EPERM, got {ret}"
        );
    }

    #[test]
    fn test_setsid_already_session_leader() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        fn handle_pid(pid: PidT) {
            if pid == 0 {
                let first = setsid();
                assert!(
                    first > 0,
                    "expected first setsid to succeed, got {first}"
                );
                assert_eq!(first, getpid());
                assert_eq!(getpgrp(), first);

                let second = setsid();
                assert_eq!(
                    second, -1,
                    "expected EPERM from repeated setsid, got {second}"
                );
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
