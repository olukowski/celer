use celer_system_linux_ctypes::{Int, Long};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`fcntl`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FcntlError {
    /// `EBADF`.
    Ebadf,
    /// `EFAULT`.
    Efault,
    /// `EINVAL`.
    Einval,
    /// `EMFILE`.
    Emfile,
    /// `EPERM`.
    Eperm,
    /// Another errno returned by delegated security-hook or file-specific
    /// command handling.
    Other(Errno),
}

impl FcntlError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Emfile => Self::Emfile,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Manipulate an open file descriptor through `fcntl(2)`.
///
/// This wrapper keeps the raw `cmd` and `arg` integers, but maps the raw
/// syscall return value into `Result<Long, FcntlError>`.
///
/// On success, returns the syscall's nonnegative result. Depending on `cmd`,
/// that may be `0`, a duplicated file descriptor, or another command-specific
/// scalar value.
///
/// See [`sys::fcntl`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// - Some `cmd` values cause the kernel to treat `arg` as a userspace pointer
///   and copy to or from that address. The caller must uphold the
///   command-specific pointer validity requirements in those cases.
///
/// # Errors
/// - [`FcntlError::Ebadf`]: `fd` is not open, or this command is rejected for
///   an `O_PATH` descriptor on the current kernel path.
/// - [`FcntlError::Efault`]: the kernel could not copy a command-specific user
///   buffer.
/// - [`FcntlError::Einval`]: `cmd` was not recognized, or a command-specific
///   integer argument was invalid.
/// - [`FcntlError::Emfile`]: a duplication command could not allocate a new
///   descriptor.
/// - [`FcntlError::Eperm`]: the requested operation was not allowed.
/// - [`FcntlError::Other`]: another security-hook or file-specific errno.
pub unsafe fn fcntl(fd: Int, cmd: Int, arg: Long) -> Result<Long, FcntlError> {
    // SAFETY: the caller must uphold any command-specific pointer preconditions
    // that apply when `arg` names userspace memory.
    let ret = unsafe { sys::fcntl(fd as _, cmd, arg) };

    result_from_ret(ret as isize, |ret| ret as Long, FcntlError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Int, Long};

    use crate::{Errno, close};

    use super::{FcntlError, fcntl};

    const F_DUPFD: Int = 0;
    const F_GETFD: Int = 1;
    const F_SETFD: Int = 2;
    const FD_CLOEXEC: Int = 1;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_fcntl_{now}"));
        path
    }

    #[test]
    fn test_fcntl_get_and_set_fd_flags() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();

        // SAFETY: `F_GETFD` treats `arg` as a scalar.
        let original_flags = unsafe { fcntl(fd, F_GETFD, 0) }.unwrap();
        // SAFETY: `F_SETFD` treats `arg` as a scalar bitmask.
        assert_eq!(
            unsafe { fcntl(fd, F_SETFD, original_flags | FD_CLOEXEC as Long) },
            Ok(0)
        );
        // SAFETY: `F_GETFD` treats `arg` as a scalar.
        let updated_flags = unsafe { fcntl(fd, F_GETFD, 0) }.unwrap();
        assert_eq!(updated_flags & FD_CLOEXEC as Long, FD_CLOEXEC as Long);

        assert_eq!(close(fd), Ok(()));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fcntl_dupfd_returns_new_descriptor() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        // SAFETY: `F_DUPFD` treats `arg` as the minimum new descriptor number.
        let dup_fd =
            unsafe { fcntl(fd, F_DUPFD, 0) }.expect("F_DUPFD should succeed");
        assert_ne!(dup_fd, fd as Long);

        assert_eq!(close(dup_fd as Int), Ok(()));
        assert_eq!(close(fd), Ok(()));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fcntl_ebadf() {
        // SAFETY: `F_GETFD` treats `arg` as a scalar.
        assert_eq!(unsafe { fcntl(-1, F_GETFD, 0) }, Err(FcntlError::Ebadf));
    }

    #[test]
    fn test_fcntl_error_mapping() {
        assert_eq!(FcntlError::from_errno(Errno::Ebadf), FcntlError::Ebadf);
        assert_eq!(FcntlError::from_errno(Errno::Efault), FcntlError::Efault);
        assert_eq!(FcntlError::from_errno(Errno::Einval), FcntlError::Einval);
        assert_eq!(FcntlError::from_errno(Errno::Emfile), FcntlError::Emfile);
        assert_eq!(FcntlError::from_errno(Errno::Eperm), FcntlError::Eperm);
        assert_eq!(
            FcntlError::from_errno(Errno::Enoent),
            FcntlError::Other(Errno::Enoent)
        );
    }
}
