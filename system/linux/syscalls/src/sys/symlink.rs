use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall2};

/// Create a symbolic link at `newname` whose stored target text is `oldname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 1.0 calls `do_symlink()` directly after two
///   `getname()` copies; current kernels route through `filename_symlinkat()`
///   and `vfs_symlink()`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Linux 1.0 copies both user strings with `getname()`, then calls
///   `do_symlink(from, to)`.
/// - The kernel resolves parent directories of `newname`, but does not
///   resolve the final component because that name is created by the syscall.
/// - The `oldname` string is passed through as the symlink target text; the
///   syscall does not path-walk it during creation.
///
/// # Errors
/// - `EFAULT`, `ENAMETOOLONG`, `ENOENT`, and `ENOMEM` are reachable from the
///   Linux 1.0 `getname()` copies at syscall entry.
/// - `ENOTDIR`, `EROFS`, `EACCES`, and `EPERM` are reachable from Linux 1.0
///   parent-directory resolution and `do_symlink()` checks.
/// - Linux 1.0 filesystem `->symlink` implementations add additional
///   reachable errors after `do_symlink()` dispatches to them; verified
///   examples include `EEXIST`, `ENOSPC`, and NFS-specific `ENAMETOOLONG`.
///
/// Current kernels may also return additional VFS, LSM, and filesystem-
/// specific errors after the generic entry path creates the destination dentry.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/symlink.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v7.0#n5669)
/// - LTS: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v7.0#n5669)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n597)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.12#n767)
pub fn symlink(oldname: *const Char, newname: *const Char) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall2(
            Sysno::Symlink,
            oldname.addr() as isize,
            newname.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::symlink;

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
    fn test_symlink_stores_target_bytes() {
        let new_path = create_temp_path("test_symlink_new");
        let old_bytes = b"missing_symlink_target".to_vec();

        let mut old_c = old_bytes.clone();
        old_c.push(0);
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        let rc = symlink(
            old_c.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert_eq!(rc, 0, "symlink failed: {rc}");

        let target = fs::read_link(&new_path).unwrap();
        assert_eq!(target.as_os_str().as_encoded_bytes(), old_bytes);

        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_symlink_existing_destination() {
        let new_path = create_temp_path("test_symlink_exists");
        fs::write(&new_path, b"occupied").unwrap();

        let old_bytes = b"missing_symlink_target\0";
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        let rc = symlink(
            old_bytes.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert_eq!(rc, -17, "symlink should fail with EEXIST: {rc}");

        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_symlink_empty_target() {
        let new_path = create_temp_path("test_symlink_empty_target");
        let old_bytes = b"\0";
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        let rc = symlink(
            old_bytes.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert_eq!(rc, -2, "symlink should fail with ENOENT: {rc}");
    }

    #[test]
    fn test_symlink_empty_destination_basename_fails() {
        let dir_path = create_temp_path("test_symlink_empty_basename_dir");
        fs::create_dir(&dir_path).unwrap();

        let old_bytes = b"missing_symlink_target\0";
        let mut new_bytes = dir_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(b'/');
        new_bytes.push(0);

        let rc = symlink(
            old_bytes.as_ptr().cast::<Char>(),
            new_bytes.as_ptr().cast::<Char>(),
        );

        assert!(rc < 0, "symlink with empty destination basename succeeded");

        fs::remove_dir(&dir_path).unwrap();
    }
}
