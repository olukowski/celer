use celer_system_linux_ctypes::{Int, OldGidT};

use crate::arch::current::{Sysno, syscall1};

/// Set the calling process's group ID.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 uses the legacy 16-bit `setgid` entry
///   (`sys_setgid16`) which forwards to the core credential logic
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None when the requested group matches the caller's real or saved group.
/// - `CAP_SETGID` is required for other transitions, subject to namespace and
///   LSM checks.
///
/// # Behavior
/// - Legacy i386 syscall `46` uses the 16-bit `old_gid_t` entry point
///   (`sys_setgid16`), which converts the old gid format before applying the
///   core rules.
/// - With `CAP_SETGID`, the kernel sets `gid`, `egid`, `sgid`, and `fsgid`.
/// - Without `CAP_SETGID`, a successful call only updates `egid` and `fsgid`,
///   and only when the requested gid matches the caller's current real or
///   saved gid.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EINVAL`: the requested gid is not valid in the caller's user namespace.
/// - `ENOMEM`: the kernel could not allocate a new credential structure.
/// - `EPERM`: the caller lacks permission for the requested transition.
/// - LSM-specific negative errors may also be returned.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setgid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n474)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n474)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n61)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n72)
pub fn setgid(gid: OldGidT) -> Int {
    // SAFETY: `setgid` takes a single integer argument and has no pointer
    // validity requirements.
    unsafe { syscall1(Sysno::Setgid, gid as isize) as Int }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::OldGidT;

    use super::setgid;

    #[test]
    fn test_setgid_invalid_gid() {
        let result = setgid(!0 as OldGidT);

        assert!(result < 0, "setgid should have failed: {result}");
    }
}
