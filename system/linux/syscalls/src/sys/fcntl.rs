use celer_system_linux_ctypes::{Int, Long, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Manipulate an open file descriptor `fd` with the legacy x86 32-bit
/// `fcntl` syscall entrypoint.
///
/// # Safety
/// - Some `cmd` values cause the kernel to treat `arg` as a userspace pointer
///   and copy to or from that address. Callers must uphold the command-specific
///   pointer validity requirements in those cases.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: command-specific support has grown over time
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Request-dependent; many commands require no privilege, but some
///   operations can require capabilities or ownership checks.
///
/// # Behavior
/// - Validates `fd` and dispatches `cmd` through the kernel's `fcntl` entry
///   path.
/// - The meaning of `arg` depends on `cmd`; it may be an integer, flags, or a
///   userspace pointer value passed to lower-level helpers.
/// - On 32-bit x86, some lock-related commands are handled by `fcntl64`
///   instead of this syscall.
/// - Some commands return state values instead of `0`, including `F_GETFD` and
///   `F_GETFL`.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `EINVAL`: `cmd` is not recognized, or the command-specific arguments are
///   invalid.
/// - `EMFILE`: `F_DUPFD` / `F_DUPFD_CLOEXEC` cannot allocate a descriptor at or
///   above the requested lower bound.
/// - `EFAULT`: a command-specific copy to or from user memory failed.
///
/// Other command-specific errors may occur, including `EPERM`, `ESRCH`,
/// `ENOLCK`, `EAGAIN`, `EACCES`, `ENOMEM`, `EEXIST`, `EBUSY`, and `EOVERFLOW`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fcntl.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/fcntl.c?h=v6.19#n587)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/fcntl.c?h=v6.18.18#n587)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/fcntl.c?h=1.0#n66)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/fcntl.c?h=0.10#n47)
pub unsafe fn fcntl(fd: UnsignedInt, cmd: Int, arg: Long) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall3(Sysno::Fcntl, fd as isize, cmd as isize, arg as isize) })
        as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::FromRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Int, Long, UnsignedInt};

    use crate::{arch::current::Sysno, sys::test_support::open};

    use super::fcntl;

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_fcntl_{now}"));

        path
    }

    #[test]
    fn test_fcntl_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Fcntl as isize, 55);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Fcntl as isize, 25);
    }

    #[test]
    fn test_fcntl_getfd() {
        let path = create_temp_path();
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let fd = unsafe {
            open(path_bytes.as_ptr().cast(), 0 as Int, 0 as libc::mode_t)
        };
        assert!(fd >= 0, "open failed: {fd}");

        // SAFETY: `F_GETFD` treats `arg` as a scalar.
        let ret = unsafe { fcntl(fd as UnsignedInt, 1 as Int, 0 as Long) };
        assert_eq!(ret, 0);

        // SAFETY: `fd` was returned by `open` above and is uniquely owned here.
        let _ = unsafe { std::fs::File::from_raw_fd(fd as _) };

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fcntl_invalid_fd() {
        // SAFETY: `F_GETFD` treats `arg` as a scalar.
        let ret = unsafe { fcntl(9_999 as _, 1 as Int, 0 as Long) };
        assert_eq!(ret, -(9 as Long));
    }
}
