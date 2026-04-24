use celer_system_linux_ctypes::{Int, OldUidT};

use crate::arch::current::{Sysno, syscall2};

/// Set the calling process's real and/or effective user ID through the legacy
/// i386 `setreuid16` ABI.
///
/// Pass `OldUidT::MAX` for either argument to leave that ID unchanged.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 0.10 had only an internal helper, not a public
///   syscall-70 entry; current i386 keeps the legacy 16-bit `setreuid16`
///   entry, which forwards into the modern credential subsystem
/// - Availability: current x86 32-bit kernels expose the legacy
///   `sys_setreuid16` entry point
///
/// # Required Privileges
/// - None when the requested IDs match the caller's currently permitted real,
///   effective, or saved IDs under the kernel's `setreuid` rules.
/// - `CAP_SETUID` is required for other transitions, subject to namespace and
///   LSM checks.
///
/// # Behavior
/// - `OldUidT::MAX` is the legacy no-change sentinel for either argument.
/// - Linux 1.0 allows changing the real UID when the requested real UID
///   matches the caller's old real UID or current effective UID, or when the
///   caller is privileged.
/// - Linux 1.0 allows changing the effective UID when the requested effective
///   UID matches the old real UID or current effective UID, or when the caller
///   is privileged; on that path it also updates the saved UID.
/// - Current kernels use the legacy 16-bit ABI on i386, update `fsuid` to the
///   new effective UID on success, and update the saved UID when required by
///   the modern `__sys_setreuid()` rules.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EAGAIN`: the kernel could not allocate per-user accounting state for the
///   requested credential change.
/// - `EINVAL`: a non-sentinel UID does not map to a valid kernel UID in the
///   caller's current user namespace.
/// - `ENOMEM`: the kernel could not allocate a new credential structure.
/// - `EPERM`: the requested UID change is not permitted for the caller.
/// - LSM-specific negative errors may also be returned.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setreuid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v6.19#n48)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n48)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n337)
///
/// # Historical References
/// - Linux 0.10 internal helper only:
///   [kernel/sys.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n118)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn setreuid16(ruid: OldUidT, euid: OldUidT) -> Int {
    // SAFETY: `setreuid16` takes only scalar arguments and has no pointer
    // validity requirements.
    unsafe { syscall2(Sysno::Setreuid, ruid as isize, euid as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{OldUidT, PidT};

    use super::setreuid16;
    use crate::sys::{
        geteuid::geteuid16,
        getuid::getuid16,
        test_support::{_exit as exit, fork, waitpid},
    };
    fn assert_child_setreuid16(ruid: OldUidT, euid: OldUidT) {
        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {pid}");

        fn handle_pid(pid: PidT, ruid: OldUidT, euid: OldUidT) {
            if pid == 0 {
                let rc = setreuid16(ruid, euid);
                unsafe { exit(if rc == 0 { 0 } else { 1 }) };
            }
        }

        handle_pid(pid, ruid, euid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }

    #[test]
    fn test_setreuid16_current_ids_in_child() {
        let uid = getuid16();
        let euid = geteuid16();

        assert_child_setreuid16(uid, euid);
    }

    #[test]
    fn test_setreuid16_no_change_real_uid_in_child() {
        let euid = geteuid16();

        assert_child_setreuid16(OldUidT::MAX, euid);
    }

    #[test]
    fn test_setreuid16_no_change_effective_uid_in_child() {
        let uid = getuid16();

        assert_child_setreuid16(uid, OldUidT::MAX);
    }
}
