use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`fsync`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FsyncError {
    /// `EBADF`.
    Ebadf,
    /// `EINVAL`.
    Einval,
    /// `EIO`.
    Eio,
    /// Another errno returned by the underlying filesystem-specific sync work.
    Other(Errno),
}

impl FsyncError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Einval => Self::Einval,
            Errno::Eio => Self::Eio,
            errno => Self::Other(errno),
        }
    }
}

/// Flush pending file data and metadata for the open file referenced by `fd`.
///
/// This safe wrapper maps the raw `fsync(2)` return value into
/// `Result<(), FsyncError>` while keeping the file-descriptor argument as the
/// kernel-facing integer type.
///
/// On success, returns `Ok(())` after the kernel has completed the requested
/// sync for `fd`.
///
/// See [`sys::fsync`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`FsyncError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FsyncError::Einval`]: `fd` refers to an object that this syscall cannot
///   sync, such as a pipe on current kernels.
/// - [`FsyncError::Eio`]: Linux 1.0 mapped a nonzero filesystem `fsync`
///   callback result to this error.
/// - [`FsyncError::Other`]: another filesystem- or object-specific sync error.
pub fn fsync(fd: Int) -> Result<(), FsyncError> {
    let ret = sys::fsync(fd);

    unit_from_ret(ret as isize, FsyncError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Int;

    use crate::Errno;

    use super::{FsyncError, fsync};

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_wrap_fsync_{now}"));

        path
    }

    #[test]
    fn test_fsync_ok() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        file.write_all(b"hello wrapped fsync").unwrap();

        assert_eq!(fsync(file.as_raw_fd() as Int), Ok(()));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fsync_ebadf() {
        assert_eq!(fsync(-1), Err(FsyncError::Ebadf));
    }

    #[test]
    fn test_fsync_einval_for_pipe() {
        let mut fds = [0 as Int; 2];

        // SAFETY: `fds` is writable for two `Int` values.
        let rc = unsafe { crate::sys::test_support::pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");

        assert_eq!(fsync(fds[0]), Err(FsyncError::Einval));

        assert_eq!(crate::sys::close(fds[0]), 0);
        assert_eq!(crate::sys::close(fds[1]), 0);
    }

    #[test]
    fn test_fsync_error_mapping() {
        assert_eq!(FsyncError::from_errno(Errno::Ebadf), FsyncError::Ebadf);
        assert_eq!(FsyncError::from_errno(Errno::Einval), FsyncError::Einval);
        assert_eq!(FsyncError::from_errno(Errno::Eio), FsyncError::Eio);
        assert_eq!(
            FsyncError::from_errno(Errno::Enomem),
            FsyncError::Other(Errno::Enomem)
        );
    }
}
