use celer_system_linux_ctypes::{Char, Long, UModeT};

use crate::arch::current::{Sysno, syscall2};

/// Create a directory named by `pathname`.
///
/// # Safety
/// - `pathname` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `mode` must be a valid directory mode bit pattern.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 0.10 required superuser privileges; Linux 1.0
///   and later use ordinary directory permission checks
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel copies the pathname and calls
///   `do_mkdirat(AT_FDCWD, getname(pathname), mode)` in the current
///   implementation.
/// - The final pathname component is created as a directory.
///
/// # Errors
/// - The kernel may return pathname-resolution, permission, and VFS errors
///   such as `ENOENT`, `EEXIST`, `ENOTDIR`, and `EPERM`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/mkdir.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n5198)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n4506)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n471)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.10#n459)
pub unsafe fn mkdir(pathname: *const Char, mode: UModeT) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Mkdir, pathname.addr() as isize, mode as isize) as Long
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, UModeT};

    use super::mkdir;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_mkdir_{now}"));

        path
    }

    #[test]
    fn test_mkdir() {
        let path = create_temp_path();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall.
        let rc = unsafe {
            mkdir(path_bytes.as_ptr().cast::<Char>(), 0o700 as UModeT)
        };

        assert_eq!(rc, 0, "mkdir failed: {rc}");
        assert!(path.exists(), "mkdir did not create the directory");
        assert!(fs::metadata(&path).unwrap().is_dir());

        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_mkdir_exists() {
        let path = create_temp_path();
        fs::create_dir(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = unsafe {
            mkdir(path_bytes.as_ptr().cast::<Char>(), 0o700 as UModeT)
        };

        assert_eq!(rc, -17, "mkdir should fail with EEXIST: {rc}");

        fs::remove_dir(&path).unwrap();
    }
}
