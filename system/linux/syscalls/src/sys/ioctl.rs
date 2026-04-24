use celer_system_linux_ctypes::{Long, UnsignedInt, UnsignedLong};

use crate::arch::current::{Sysno, syscall3};

/// Perform an `ioctl` operation on the open file descriptor `fd`.
///
/// # Safety
/// - Some `request` values cause the kernel to treat `arg` as a userspace
///   pointer and copy to or from that address. Callers must uphold the
///   request-specific pointer validity requirements in those cases.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Request-dependent; many `ioctl` commands require no privilege, but some
///   built-in operations require capabilities such as `CAP_SYS_RAWIO` or
///   `CAP_SYS_ADMIN`.
///
/// # Behavior
/// - Validates `fd`, runs the security hook, then dispatches the request
///   through the kernel's generic `ioctl` path.
/// - The request may be handled by built-in VFS helpers or by the file
///   object's `ioctl` handler.
/// - On success, returns the handler's result, which is often `0` but may be
///   command-specific.
/// - The kernel does not interpret whether `arg` is an integer or a pointer;
///   that depends on `request`.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `ENOTTY`: the request is unsupported for the target file object or by
///   the generic VFS path.
/// - Other command-specific errors may occur, including `EFAULT`, `EINVAL`,
///   `EPERM`, `EOPNOTSUPP`, `ERANGE`, `EBADR`, and `ENOMEM`.
///
/// The kernel validates the descriptor and runs the security hook before
/// dispatch. Specific `ioctl` commands may copy to or from user memory and can
/// return additional helper-specific errors.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/ioctl.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/ioctl.c?h=v7.0#n583)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/ioctl.c?h=v6.18.18#n583)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/ioctl.c?h=1.0#n57)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/ioctl.c?h=0.10#n30)
pub unsafe fn ioctl(
    fd: UnsignedInt,
    request: UnsignedLong,
    arg: UnsignedLong,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(Sysno::Ioctl, fd as isize, request as isize, arg as isize)
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Long, UnsignedLong};

    use crate::arch::current::Sysno;

    use super::ioctl;

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_ioctl_{now}"));

        path
    }

    #[test]
    fn test_ioctl_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 54;
        #[cfg(target_arch = "aarch64")]
        let expected = 29;
        #[cfg(target_arch = "x86_64")]
        let expected = 16;

        assert_eq!(Sysno::Ioctl as isize, expected);
    }

    #[test]
    fn test_ioctl_invalid_fd() {
        // SAFETY: request `0` treats `arg` as a scalar here.
        let ret =
            unsafe { ioctl(9_999 as _, 0 as UnsignedLong, 0 as UnsignedLong) };
        assert!(
            ret < 0,
            "ioctl on an invalid fd unexpectedly succeeded: {ret}"
        );
    }

    #[test]
    fn test_ioctl_enotty_on_regular_file() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();

        // SAFETY: this request treats `arg` as a scalar and does not require a
        // userspace pointer.
        let ret = unsafe {
            ioctl(
                file.as_raw_fd() as _,
                0xDEAD_BEEF as UnsignedLong,
                0 as UnsignedLong,
            )
        };
        assert_eq!(ret, -(25 as Long));

        fs::remove_file(&path).unwrap();
    }
}
