use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall0};

/// Hang up the calling process's controlling terminal.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 checked `suser()` and hung up
///   `current->tty` directly; current kernels require `CAP_SYS_TTY_CONFIG`
///   and delegate the hangup through `tty_vhangup_self()`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser.
/// - Current kernels: the caller must have `CAP_SYS_TTY_CONFIG`.
///
/// # Behavior
/// - On success, the kernel hangs up the caller's controlling terminal.
/// - If the caller has no controlling terminal, the syscall succeeds and does
///   nothing.
/// - Returns `0` on success.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to invoke `vhangup`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/vhangup.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v7.0#n1528)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n1608)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n482)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn vhangup() -> Int {
    // SAFETY: `vhangup` takes no arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall0(Sysno::Vhangup) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PidT};

    use crate::{
        arch::current::{Sysno, syscall0},
        sys::{exit, fork, setsid, waitpid},
    };

    use super::vhangup;

    const EPERM: Int = -(1 as Int);

    #[test]
    fn test_vhangup_sysno() {
        assert_eq!(Sysno::Vhangup as isize, 111);
    }

    #[test]
    fn test_vhangup_after_setsid_matches_raw_syscall() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");

        fn handle_child(pid: PidT) {
            if pid == 0 {
                let sid = setsid();
                assert!(
                    sid > 0,
                    "setsid should succeed in the child, got {sid}"
                );

                let wrapped = vhangup();
                let raw = unsafe { syscall0(Sysno::Vhangup) as Int };

                assert_eq!(
                    wrapped, raw,
                    "vhangup wrapper should match raw syscall"
                );
                // After `setsid()` the child should have no controlling tty,
                // but an unprivileged test process still stops at `EPERM`
                // before the kernel reaches that success path.
                assert!(
                    wrapped == 0 || wrapped == EPERM,
                    "expected success or EPERM from vhangup without a controlling tty, got {wrapped}",
                );
                exit(0);
            }
        }

        handle_child(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }
}
