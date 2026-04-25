use celer_system_linux_ctypes::{Long, UnsignedInt, UnsignedLong};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`ioctl`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IoctlError {
    /// `EBADF`.
    Ebadf,
    /// `ENOTTY`.
    Enotty,
    /// Another errno returned by request-specific or delegated handling.
    Other(Errno),
}

impl IoctlError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Enotty => Self::Enotty,
            errno => Self::Other(errno),
        }
    }
}

/// Perform an `ioctl(2)` operation on an open file descriptor.
///
/// This wrapper preserves the raw `request` and `arg` integers, but maps the
/// raw syscall return value into `Result<Long, IoctlError>`.
///
/// On success, returns the nonnegative result reported by the selected ioctl
/// handler. Many commands return `0`, but request-specific handlers may return
/// other scalar values.
///
/// See [`sys::ioctl`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// - Some `request` values cause the kernel to interpret `arg` as a userspace
///   pointer and copy to or from that address. The caller must uphold the
///   request-specific pointer validity requirements in those cases.
///
/// # Errors
/// - [`IoctlError::Ebadf`]: `fd` does not refer to an open file descriptor.
/// - [`IoctlError::Enotty`]: the target file object does not support
///   `request`.
/// - [`IoctlError::Other`]: another request-specific, security-hook, or
///   object-specific errno.
pub unsafe fn ioctl(
    fd: UnsignedInt,
    request: UnsignedLong,
    arg: UnsignedLong,
) -> Result<Long, IoctlError> {
    // SAFETY: the caller must uphold any request-specific pointer validity
    // requirements when `arg` names userspace memory.
    let ret = unsafe { sys::ioctl(fd, request, arg) };

    result_from_ret(ret as isize, |ret| ret as Long, IoctlError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UnsignedLong;

    use crate::Errno;

    use super::{IoctlError, ioctl};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_ioctl_{now}"));
        path
    }

    #[test]
    fn test_ioctl_invalid_fd() {
        // SAFETY: request `0` treats `arg` as a scalar in this test.
        assert_eq!(
            unsafe { ioctl(9_999, 0 as UnsignedLong, 0 as UnsignedLong) },
            Err(IoctlError::Ebadf)
        );
    }

    #[test]
    fn test_ioctl_enotty_on_regular_file() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();

        // SAFETY: this bogus request treats `arg` as a scalar and is intended
        // to exercise the unsupported-command path.
        let err = unsafe {
            ioctl(file.as_raw_fd() as _, 0xDEAD_BEEF, 0 as UnsignedLong)
        }
        .expect_err("a regular file should reject an unsupported ioctl");

        assert_eq!(err, IoctlError::Enotty);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_ioctl_error_mapping() {
        assert_eq!(IoctlError::from_errno(Errno::Ebadf), IoctlError::Ebadf);
        assert_eq!(IoctlError::from_errno(Errno::Enotty), IoctlError::Enotty);
        assert_eq!(
            IoctlError::from_errno(Errno::Efault),
            IoctlError::Other(Errno::Efault)
        );
    }
}
