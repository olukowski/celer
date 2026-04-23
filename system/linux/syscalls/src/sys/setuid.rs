use celer_system_linux_ctypes::{Int, OldUidT};

use crate::arch::current::{Sysno, syscall1};

/// Set the calling process's user ID through the legacy i386 `setuid16` ABI.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 uses the legacy 16-bit `setuid` entry
///   (`sys_setuid16`) which forwards to the core credential logic
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None when the requested user matches the caller's real or saved user.
/// - `CAP_SETUID` is required for other transitions, subject to namespace and
///   LSM checks.
///
/// # Behavior
/// - Legacy i386 syscall `23` uses the 16-bit `old_uid_t` entry point
///   (`sys_setuid16`), which converts the old uid format before applying the
///   core rules.
/// - With `CAP_SETUID`, the kernel sets `uid`, `euid`, `suid`, and `fsuid`.
/// - Without `CAP_SETUID`, a successful call only updates `euid` and `fsuid`,
///   and only when the requested uid matches the caller's current real or
///   saved uid.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EINVAL`: `uid` cannot be mapped to a valid kernel UID.
/// - `EPERM`: the caller is not permitted to switch to `uid`.
/// - `ENOMEM`: on current kernels, credential allocation fails before the uid
///   transition completes.
/// - Current kernels may also return additional policy-dependent errors from
///   the credential and LSM hooks that run after the initial uid conversion.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setuid.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n53)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n53)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n374)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n143)
pub fn setuid16(uid: OldUidT) -> Int {
    // SAFETY: `setuid16` takes a single integer argument and has no pointer
    // validity requirements.
    unsafe { syscall1(Sysno::Setuid, uid as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldUidT;

    use crate::arch::current::{Sysno, syscall0};

    use super::setuid16;

    #[test]
    fn test_setuid16_current_uid() {
        let uid = unsafe { syscall0(Sysno::Getuid) as OldUidT };
        let result = setuid16(uid);

        assert_eq!(result, 0, "setuid16(current uid) failed: {result}");
    }
}
