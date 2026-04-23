use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Enable or disable process accounting through the historical `acct` syscall
/// slot.
///
/// Linux 1.0 exposed syscall number `51` as `acct`, but its entrypoint was a
/// stub that always returned `-ENOSYS`. Current kernels implement process
/// accounting on that same slot.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes:
///   - Linux 1.0 returned `-ENOSYS` unconditionally from the syscall
///     entrypoint.
///   - Current kernels enable accounting on the named file, or disable it when
///     `name` is null.
/// - Availability: present on supported Linux kernels
///
/// # Required Privileges
/// - Linux 1.0: none are reachable, because the syscall entrypoint always
///   returns `-ENOSYS`.
/// - Current kernels require `CAP_SYS_PACCT`.
///
/// # Behavior
/// - On Linux 1.0, the syscall does not inspect `name`.
/// - On Linux 1.0, the kernel does not enable or disable accounting.
/// - On Linux 1.0, every call returns `-ENOSYS`.
/// - On current kernels, a null `name` disables accounting for the current PID
///   namespace.
/// - On current kernels, a non-null `name` enables accounting on the named
///   file.
/// - Current kernels reject callers without `CAP_SYS_PACCT` before consulting
///   the path.
///
/// # Errors
/// - `ENOSYS`: always returned by the Linux 1.0 syscall entrypoint.
/// - `EPERM`: on current kernels, the caller lacks `CAP_SYS_PACCT`.
/// - Current kernels may also return pathname- and file-handling errors when
///   `name` is non-null.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/acct.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/acct.c?h=v6.19#n293)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/acct.c?h=v6.18.18#n293)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n294)
pub unsafe fn acct(name: *const Char) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Acct, name.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::acct;

    fn create_temp_path() -> Vec<u8> {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_acct_{now}"));

        let mut bytes = path.as_os_str().as_encoded_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    #[test]
    fn test_acct_invalid_path() {
        let path = create_temp_path();

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let result = unsafe { acct(path.as_ptr().cast::<Char>()) };

        assert!(result < 0, "acct should have failed: {result}");
    }
}
