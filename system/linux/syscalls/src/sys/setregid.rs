use celer_system_linux_ctypes::{Int, OldGidT};

use crate::arch::current::{Sysno, syscall2};

/// Set the calling process's real and/or effective group ID through the legacy
/// i386 `setregid16` ABI.
///
/// Pass `OldGidT::MAX` for either argument to leave that ID unchanged.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 0.10 had only an internal helper, not a public
///   syscall-71 entry; current i386 keeps the legacy 16-bit `setregid16`
///   entry when `CONFIG_UID16` is enabled, forwarding into the modern
///   credential subsystem
/// - Availability: current x86 32-bit kernels route syscall 71 to the legacy
///   `setregid16` path when `CONFIG_UID16` is enabled; otherwise the slot can
///   fall back to `sys_ni_syscall`
///
/// # Required Privileges
/// - None when the requested IDs match the caller's currently permitted real,
///   effective, or saved IDs under the kernel's `setregid` rules.
/// - `CAP_SETGID` is required for other transitions, subject to namespace and
///   LSM checks.
///
/// # Behavior
/// - `OldGidT::MAX` is the legacy no-change sentinel for either argument.
/// - Linux 1.0 allows changing the real GID when the requested real GID
///   matches the caller's old real GID or current effective GID, or when the
///   caller is privileged.
/// - Linux 1.0 allows changing the effective GID when the requested effective
///   GID matches the old real GID or current effective GID, or when the caller
///   is privileged; on that path it also updates the saved GID.
/// - Current kernels use the legacy 16-bit ABI on i386, update `fsgid` to the
///   new effective GID on success, and update the saved GID when required by
///   the modern `__sys_setregid()` rules.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EINVAL`: a non-sentinel GID does not map to a valid kernel GID in the
///   caller's current user namespace.
/// - `ENOMEM`: the kernel could not allocate a new credential structure.
/// - `EPERM`: the requested GID change is not permitted for the caller.
/// - `ENOSYS`: builds without `CONFIG_UID16` may route syscall 71 to
///   `sys_ni_syscall` instead of `setregid16`.
/// - LSM-specific negative errors may also be returned.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setregid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v6.19#n38)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n38)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n254)
///
/// # Historical References
/// - Linux 0.10 internal helper only:
///   [kernel/sys.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n51)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn setregid16(rgid: OldGidT, egid: OldGidT) -> Int {
    // SAFETY: `setregid16` takes only scalar arguments and has no pointer
    // validity requirements.
    unsafe { syscall2(Sysno::Setregid, rgid as isize, egid as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, OldGidT, PidT};

    use super::setregid16;
    use crate::sys::{exit, fork, getegid16, getgid16, waitpid};
    fn assert_child_setregid16(
        rgid: OldGidT,
        egid: OldGidT,
        expected_gid: OldGidT,
        expected_egid: OldGidT,
    ) {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");

        const ENOSYS: Int = -38;

        fn handle_pid(
            pid: PidT,
            rgid: OldGidT,
            egid: OldGidT,
            expected_gid: OldGidT,
            expected_egid: OldGidT,
        ) {
            if pid == 0 {
                let rc = setregid16(rgid, egid);
                if rc == ENOSYS {
                    exit(0);
                }

                let gid = getgid16();
                let egid_now = getegid16();
                let ok =
                    rc == 0 && gid == expected_gid && egid_now == expected_egid;
                exit(if ok { 0 } else { 1 });
            }
        }

        handle_pid(pid, rgid, egid, expected_gid, expected_egid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }

    #[test]
    fn test_setregid16_current_ids_in_child() {
        let gid = getgid16();
        let egid = getegid16();

        assert_child_setregid16(gid, egid, gid, egid);
    }

    #[test]
    fn test_setregid16_no_change_real_gid_in_child() {
        let gid = getgid16();
        let egid = getegid16();

        assert_child_setregid16(OldGidT::MAX, egid, gid, egid);
    }

    #[test]
    fn test_setregid16_no_change_effective_gid_in_child() {
        let gid = getgid16();
        let egid = getegid16();

        assert_child_setregid16(gid, OldGidT::MAX, gid, egid);
    }
}
