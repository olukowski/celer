use celer_system_linux_ctypes::{Char, Int, Long};

use crate::arch::current::{Sysno, syscall2};

/// Check whether the calling process can access a file by pathname.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None.
/// - When `AT_EACCESS` is not in play, the kernel may temporarily override the
///   subjective credentials to the caller's real `uid`/`gid` and adjust
///   capabilities before checking access.
///
/// # Behavior
/// - `mode` is validated as the traditional `F_OK`/`X_OK`/`W_OK`/`R_OK` mask.
/// - On success, returns `0` if the requested access is permitted.
/// - This wrapper uses the plain pathname `access` ABI, not `faccessat2`.
///
/// # Errors
/// - `EINVAL`: `mode` contains unsupported bits.
/// - `ENOMEM`: temporary override credential allocation failed.
/// - `EACCES`, `EPERM`, `EROFS`, `ENOENT`, and other pathname or permission
///   errors may be returned by the VFS path and inode permission checks.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/access.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n547)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n547)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n47)
pub unsafe fn access(pathname: *const Char, mode: Int) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Access, pathname.addr() as isize, mode as isize)
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, File},
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::access;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_sys_access_{now}"));
        path
    }

    #[test]
    fn test_access() {
        let path = temp_path();
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe { access(path_bytes.as_ptr().cast::<Char>(), 0) };
        assert_eq!(ret, 0, "access failed: {ret}");

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let invalid =
            unsafe { access(path_bytes.as_ptr().cast::<Char>(), 0x8000) };
        assert_eq!(
            invalid, -22,
            "expected EINVAL from invalid mode, got {invalid}"
        );

        fs::remove_file(&path).unwrap();
    }
}
