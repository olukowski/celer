use celer_system_linux_ctypes::{Char, Long, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Set the size of the file named by `path` to `length` bytes.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: current kernels use a signed `long` truncate ABI, so
///   this historical wrapper's `length` values above `LONG_MAX` on x86 are
///   rejected with `EINVAL` before pathname lookup
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for the syscall itself.
/// - The target must still be writable, and truncation on a read-only
///   filesystem is rejected by the kernel.
///
/// # Behavior
/// - Linux 1.0 resolves `path` through `namei()`, so the final pathname
///   component is followed if it is a symbolic link.
/// - On success, Linux 1.0 sets the inode size to `length`, calls the
///   filesystem truncate hook when present, updates `ctime` and `mtime`, and
///   marks the inode dirty before returning.
/// - The historical syscall ABI takes `length` as an unsigned integer.
///
/// # Errors
/// - `EFAULT`: `path` is not a valid user pathname pointer.
/// - `ENAMETOOLONG`: `path` does not fit in the single-page `getname()`
///   buffer used by the Linux 1.0 entry path.
/// - `ENOENT`: `path` is empty, or pathname lookup otherwise fails with
///   `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer.
/// - `ENOTDIR`: pathname traversal required a directory but encountered a
///   non-directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory, or
///   the resolved inode is a directory or is not writable.
/// - `EROFS`: the resolved inode is on a read-only filesystem.
/// - `EINVAL`: on current kernels, `length` values above `LONG_MAX` on x86
///   cross the sign bit when this historical ABI is forwarded to the modern
///   signed-length syscall entry.
///
/// Linux 1.0 may also propagate additional filesystem-specific errors from
/// symlink following and from `notify_change()`. This wrapper does not
/// enumerate them because they are not fully verifiable from the generic
/// syscall entry path alone.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/truncate.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v7.0#n152)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n151)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n68)
pub fn truncate(path: *const Char, length: UnsignedInt) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall2(Sysno::Truncate, path.addr() as isize, length as isize)
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        io::Write as _,
        os::unix::fs::symlink,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, UnsignedInt};

    use crate::arch::current::Sysno;

    use super::truncate;

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
    fn test_truncate_syscall_number() {
        assert_eq!(Sysno::Truncate as isize, 92);
    }

    #[test]
    fn test_truncate_shrinks_regular_file() {
        let path = create_temp_path("celer_sys_truncate_file");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"truncate-me").unwrap();
        drop(file);

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = truncate(path_bytes.as_ptr().cast::<Char>(), 4 as UnsignedInt);

        assert_eq!(rc, 0, "truncate failed: {rc}");
        assert_eq!(fs::metadata(&path).unwrap().len(), 4);
        assert_eq!(fs::read(&path).unwrap(), b"trun");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_truncate_follows_symlink() {
        let target_path = create_temp_path("celer_sys_truncate_target");
        let link_path = create_temp_path("celer_sys_truncate_link");
        let mut file = fs::File::create(&target_path).unwrap();
        file.write_all(b"abcdef").unwrap();
        drop(file);
        symlink(&target_path, &link_path).unwrap();

        let mut link_bytes = link_path.as_os_str().as_encoded_bytes().to_vec();
        link_bytes.push(0);

        let rc = truncate(link_bytes.as_ptr().cast::<Char>(), 2 as UnsignedInt);

        assert_eq!(rc, 0, "truncate through symlink failed: {rc}");
        assert_eq!(fs::metadata(&target_path).unwrap().len(), 2);
        assert_eq!(fs::read(&target_path).unwrap(), b"ab");
        assert!(
            fs::symlink_metadata(&link_path)
                .unwrap()
                .file_type()
                .is_symlink()
        );

        fs::remove_file(&link_path).unwrap();
        fs::remove_file(&target_path).unwrap();
    }

    #[test]
    fn test_truncate_empty_path_returns_enoent() {
        let rc = truncate(c"".as_ptr().cast::<Char>(), 1 as UnsignedInt);

        assert_eq!(rc, -2, "expected ENOENT from truncate: {rc}");
    }

    #[test]
    fn test_truncate_high_bit_length_returns_einval_on_current_kernels() {
        let rc = truncate(c"".as_ptr().cast::<Char>(), 0x8000_0000);

        assert_eq!(rc, -22, "expected EINVAL from truncate: {rc}");
    }
}
