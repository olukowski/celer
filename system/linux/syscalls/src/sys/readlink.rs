use celer_system_linux_ctypes::{Char, Int, Long};

use crate::arch::current::{Sysno, syscall3};

/// Read the stored target bytes of the symbolic link named by `path` into
/// `buf`.
///
/// # Safety
/// - If `bufsiz` is greater than `0`, `buf` must point to writable memory for
///   `bufsiz` bytes, and no live Rust references may allow the kernel's
///   writes to violate aliasing guarantees for that region during the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 0.12 read symlink contents through a built-in
///   implementation in `sys_readlink`; Linux 1.0 delegates to the resolved
///   inode's `readlink` operation
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Linux 1.0 validates the writable output range before pathname lookup.
/// - `path` is interpreted as a user pathname string; invalid or unterminated
///   pathnames are reported by the kernel as syscall errors.
/// - The final pathname component is not followed; the syscall reads the link
///   object itself.
/// - On success, returns the number of bytes copied into `buf`.
/// - The copied bytes are not NUL-terminated by the syscall.
/// - The kernel may truncate the returned link bytes to fit `bufsiz`.
///
/// # Errors
/// - `EINVAL`: `bufsiz` is less than or equal to `0`, or the resolved inode
///   does not implement `readlink`.
/// - `EFAULT`: `buf` is not a writable user range, or `path` is not a valid
///   user pathname pointer.
/// - `ENAMETOOLONG`: `path` does not fit in the single-page `getname()`
///   buffer used by the Linux 1.0 entry path.
/// - `ENOENT`: `path` is empty, names a missing component, or pathname lookup
///   otherwise fails with `ENOENT`.
/// - `ENOTDIR`: pathname traversal required a directory, but encountered a
///   non-directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer
///   used by `getname()`.
///
/// Linux 1.0 also returns filesystem-specific errors from pathname resolution
/// and from the resolved inode's `readlink` operation. This wrapper does not
/// enumerate them because they depend on the mounted filesystem
/// implementation.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/readlink.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0#n606)
/// - LTS: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v7.0#n606)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n183)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=0.12#n69)
pub unsafe fn readlink(path: *const Char, buf: *mut Char, bufsiz: Int) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(
            Sysno::Readlink,
            path.addr() as isize,
            buf.addr() as isize,
            bufsiz as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        os::unix::ffi::OsStrExt as _,
        os::unix::fs::symlink,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, Int, Long};

    use crate::arch::current::Sysno;

    use super::readlink;

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
    fn test_readlink_syscall_number() {
        assert_eq!(Sysno::Readlink as isize, 85);
    }

    #[test]
    fn test_readlink_reads_symlink_target() {
        let link_path = create_temp_path("celer_sys_readlink_link");
        let target = b"readlink_target_bytes";

        symlink(std::ffi::OsStr::from_bytes(target), &link_path).unwrap();

        let mut path_bytes = link_path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut buf = [0xA5_u8; 64];

        // SAFETY: `path_bytes` is NUL-terminated and readable for the
        // duration of the syscall, and `buf` is writable for `buf.len()`
        // bytes without overlapping mutable aliases.
        let ret = unsafe {
            readlink(
                path_bytes.as_ptr().cast::<Char>(),
                buf.as_mut_ptr().cast::<Char>(),
                buf.len() as Int,
            )
        };

        assert_eq!(ret, target.len() as Long, "readlink failed: {ret}");
        assert_eq!(&buf[..target.len()], target);
        assert_eq!(buf[target.len()], 0xA5, "readlink appended a NUL byte");

        fs::remove_file(&link_path).unwrap();
    }

    #[test]
    fn test_readlink_truncates_to_buffer_length() {
        let link_path = create_temp_path("celer_sys_readlink_truncate");
        let target = b"readlink_target_bytes";

        symlink(std::ffi::OsStr::from_bytes(target), &link_path).unwrap();

        let mut path_bytes = link_path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut buf = [0u8; 7];

        // SAFETY: `path_bytes` is NUL-terminated and readable for the
        // duration of the syscall, and `buf` is writable for `buf.len()`
        // bytes without overlapping mutable aliases.
        let ret = unsafe {
            readlink(
                path_bytes.as_ptr().cast::<Char>(),
                buf.as_mut_ptr().cast::<Char>(),
                buf.len() as Int,
            )
        };

        assert_eq!(ret, buf.len() as Long, "readlink failed: {ret}");
        assert_eq!(&buf, &target[..buf.len()]);

        fs::remove_file(&link_path).unwrap();
    }

    #[test]
    fn test_readlink_regular_file_returns_einval() {
        let path = create_temp_path("celer_sys_readlink_regular");
        fs::write(&path, b"not_a_symlink").unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);
        let mut buf = [0u8; 32];

        // SAFETY: `path_bytes` is NUL-terminated and readable for the
        // duration of the syscall, and `buf` is writable for `buf.len()`
        // bytes without overlapping mutable aliases.
        let ret = unsafe {
            readlink(
                path_bytes.as_ptr().cast::<Char>(),
                buf.as_mut_ptr().cast::<Char>(),
                buf.len() as Int,
            )
        };

        assert_eq!(ret, -(22 as Long), "expected EINVAL from readlink");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_readlink_empty_path_returns_enoent() {
        let path = b"\0";
        let mut buf = [0u8; 32];

        // SAFETY: `path` is NUL-terminated and readable for the duration of
        // the syscall, and `buf` is writable for `buf.len()` bytes without
        // overlapping mutable aliases.
        let ret = unsafe {
            readlink(
                path.as_ptr().cast::<Char>(),
                buf.as_mut_ptr().cast::<Char>(),
                buf.len() as Int,
            )
        };

        assert_eq!(ret, -(2 as Long), "expected ENOENT from readlink");
    }

    #[test]
    fn test_readlink_nonpositive_bufsiz_returns_einval() {
        let link_path = create_temp_path("celer_sys_readlink_nonpositive");
        let target = b"readlink_target_bytes";

        symlink(std::ffi::OsStr::from_bytes(target), &link_path).unwrap();

        let mut path_bytes = link_path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);
        let mut buf = [0u8; 1];

        // SAFETY: `path_bytes` is NUL-terminated and readable for the
        // duration of the syscall. `bufsiz` is `0`, so the kernel returns
        // before validating or writing through `buf`.
        let ret = unsafe {
            readlink(
                path_bytes.as_ptr().cast::<Char>(),
                buf.as_mut_ptr().cast::<Char>(),
                0,
            )
        };

        assert_eq!(ret, -(22 as Long), "expected EINVAL from readlink");

        fs::remove_file(&link_path).unwrap();
    }
}
