use celer_system_linux_ctypes::{Int, UidT};

use crate::arch::current::{Sysno, syscall1};

/// Set the calling process's user ID.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: the implementation later moved into `__sys_setuid`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for calls that do not change the process's identity.
/// - Permission checks apply when changing to a different user ID.
///
/// # Errors
/// - `EINVAL`: `uid` cannot be mapped to a valid kernel UID.
/// - `EPERM`: the caller is not permitted to switch to `uid`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setuid.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n651)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n651)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n374)
///
/// # Historical References
/// - First appearance: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n125)
pub fn setuid(uid: UidT) -> Int {
    // SAFETY: `setuid` takes a single integer argument and has no pointer
    // validity requirements.
    unsafe { syscall1(Sysno::Setuid, uid as isize) as Int }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::{Int, UidT};

    use super::setuid;

    #[test]
    fn test_setuid_invalid_uid() {
        let result = setuid(!0 as UidT);

        assert!(result < 0, "setuid should have failed: {result}");
    }
}
