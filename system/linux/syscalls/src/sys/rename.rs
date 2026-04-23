use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall2};

/// Rename a filesystem object from `oldname` to `newname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.95
/// - Behavior changes: Linux 0.10 and 0.12 carried a stub that returned
///   `-ENOSYS`; Linux 0.95 and later implement the syscall in `fs/namei.c`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel copies both pathnames and calls
///   `do_renameat2(AT_FDCWD, ..., AT_FDCWD, ..., 0)` in the current
///   implementation.
/// - The kernel does not follow the final path component when renaming.
///
/// # Errors
/// - The kernel may return pathname resolution, permission, and VFS errors
///   such as `ENOENT`, `EEXIST`, `EXDEV`, and `EPERM`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/rename.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n6097)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n5409)
/// - First confirmed implementation: [Linux 0.95](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.95#n412)
///
/// # Historical References
/// - First stub: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n41)
pub fn rename(oldname: *const Char, newname: *const Char) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe {
        syscall2(
            Sysno::Rename,
            oldname.addr() as isize,
            newname.addr() as isize,
        ) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::rename;

    fn create_temp_path(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));

        path
    }

    #[test]
    fn test_rename() {
        let old_path = create_temp_path("test_rename_old");
        let new_path = create_temp_path("test_rename_new");
        File::create(&old_path).unwrap();

        let mut old_bytes = old_path.as_os_str().as_encoded_bytes().to_vec();
        old_bytes.push(0);
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        let rc = rename(
            old_bytes.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert_eq!(rc, 0, "rename failed: {rc}");
        assert!(!old_path.exists(), "rename left the old path behind");
        assert!(new_path.exists(), "rename did not create the new path");

        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_rename_missing_source() {
        let old_path = create_temp_path("test_rename_missing_old");
        let new_path = create_temp_path("test_rename_missing_new");

        let mut old_bytes = old_path.as_os_str().as_encoded_bytes().to_vec();
        old_bytes.push(0);
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        let rc = rename(
            old_bytes.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert!(rc < 0, "rename missing source unexpectedly succeeded: {rc}");
    }
}
