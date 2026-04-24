#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::UnsignedInt;
use celer_system_linux_ctypes::{Char, Long, OffT};

use crate::arch::current::{Sysno, syscall2};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2,
};

/// Set the size of the file named by `path` to `length` bytes.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepted an unsigned length; current x86
///   kernels use a signed `off_t`-shaped ABI
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for the syscall itself.
/// - The target must still be writable, and truncation on a read-only
///   filesystem is rejected by the kernel.
///
/// # Behavior
/// - Resolves `path` to the target file, following the final component when
///   it is a symbolic link.
/// - On success, sets the inode size to `length`.
/// - This wrapper uses the current x86 `off_t`-shaped ABI for `length`, even
///   though Linux 1.0 implemented `sys_truncate` with an `unsigned int`
///   length parameter.
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
/// - `EINVAL`: on current kernels, `length` is negative.
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
pub unsafe fn truncate(path: *const Char, length: OffT) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Truncate, path.addr() as isize, length as isize)
    }) as Long
}

/// Set the size of the file named by `path` through the Linux 1.0 unsigned
/// `truncate` ABI.
///
/// This is the historical Linux 1.0 ABI at syscall slot `92`, which took
/// `length` as an `unsigned int`. Current x86 kernels use the same syscall
/// number for the signed current ABI exposed by [`truncate`].
///
/// # Safety
/// - `path` must point to a readable NUL-terminated string for the duration
///   of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n68)
/// - Current implementation:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v7.0#n152)
#[cfg(target_arch = "x86")]
pub unsafe fn truncate_1_0(path: *const Char, length: UnsignedInt) -> Long {
    // SAFETY: the wrapper forwards the raw historical ABI argument.
    (unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Truncate,
            path.addr() as isize,
            length as isize,
        )
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

    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::UnsignedInt;
    use celer_system_linux_ctypes::{Char, OffT};

    use crate::arch::current::Sysno;
    #[cfg(target_arch = "x86")]
    use crate::arch::linux_1_0::Sysno as Linux10Sysno;

    use super::truncate;
    #[cfg(target_arch = "x86")]
    use super::truncate_1_0;

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
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Truncate as isize, 92);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Truncate as isize, 45);
    }

    #[test]
    fn test_truncate_shrinks_regular_file() {
        let path = create_temp_path("celer_sys_truncate_file");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"truncate-me").unwrap();
        drop(file);

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let rc =
            unsafe { truncate(path_bytes.as_ptr().cast::<Char>(), 4 as OffT) };

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

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let rc =
            unsafe { truncate(link_bytes.as_ptr().cast::<Char>(), 2 as OffT) };

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
        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let rc = unsafe { truncate(c"".as_ptr().cast::<Char>(), 1 as OffT) };

        assert_eq!(rc, -2, "expected ENOENT from truncate: {rc}");
    }

    #[test]
    fn test_truncate_negative_length_returns_einval_on_current_kernels() {
        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let rc = unsafe { truncate(c"".as_ptr().cast::<Char>(), -1 as OffT) };

        assert_eq!(rc, -22, "expected EINVAL from truncate: {rc}");
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_truncate_1_0_syscall_number() {
        assert_eq!(Linux10Sysno::Truncate as isize, 92);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_truncate_1_0_shrinks_regular_file() {
        let path = create_temp_path("celer_sys_truncate_1_0_file");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"truncate-me").unwrap();
        drop(file);

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let rc = unsafe {
            truncate_1_0(path_bytes.as_ptr().cast::<Char>(), 4 as UnsignedInt)
        };

        assert_eq!(rc, 0, "truncate_1_0 failed: {rc}");
        assert_eq!(fs::metadata(&path).unwrap().len(), 4);
        assert_eq!(fs::read(&path).unwrap(), b"trun");

        fs::remove_file(&path).unwrap();
    }
}
