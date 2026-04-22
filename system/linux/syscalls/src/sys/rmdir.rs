use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall1};

/// Remove an empty directory named by `pathname`.
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
///   `do_rmdir(AT_FDCWD, getname(pathname))` in the current implementation.
/// - The target must be an empty directory and must not be the root directory.
///
/// # Errors
/// - The kernel may return pathname-resolution and VFS errors such as
///   `ENOENT`, `ENOTEMPTY`, `EINVAL`, `EBUSY`, and `EPERM`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/rmdir.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n5322)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n4620)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n512)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.10#n583)
pub fn rmdir(pathname: *const Char) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe { syscall1(Sysno::Rmdir, pathname.addr() as isize) as Long }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::rmdir;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_rmdir_{now}"));

        path
    }

    #[test]
    fn test_rmdir() {
        let path = create_temp_path();
        fs::create_dir(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = rmdir(path_bytes.as_ptr().cast::<Char>());

        assert_eq!(rc, 0, "rmdir failed: {rc}");
        assert!(!path.exists(), "rmdir did not remove the directory");
    }

    #[test]
    fn test_rmdir_nonempty() {
        let path = create_temp_path();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("child"), b"child").unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = rmdir(path_bytes.as_ptr().cast::<Char>());

        assert_eq!(rc, -39, "rmdir should fail with ENOTEMPTY: {rc}");

        fs::remove_file(path.join("child")).unwrap();
        fs::remove_dir(&path).unwrap();
    }
}
