use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`close`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CloseError {
    /// `EBADF`.
    Ebadf,
    /// `EINTR`.
    Eintr,
    /// Another errno returned by delegated file close or flush work.
    Other(Errno),
}

impl CloseError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Eintr => Self::Eintr,
            errno => Self::Other(errno),
        }
    }
}

/// Close an open file descriptor.
///
/// This safe wrapper maps the raw `close(2)` return value into
/// `Result<(), CloseError>` while keeping the file-descriptor argument as the
/// kernel-facing integer type.
///
/// On success, returns `Ok(())` after the kernel has closed `fd`.
///
/// See [`sys::close`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`CloseError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`CloseError::Eintr`]: the close path was interrupted.
/// - [`CloseError::Other`]: delegated file close or flush error.
pub fn close(fd: Int) -> Result<(), CloseError> {
    let ret = sys::close(fd);

    unit_from_ret(ret as isize, CloseError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{CloseError, close};

    #[test]
    fn test_close_ok() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("celer_wrap_close_{now}"));
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        assert_eq!(close(fd), Ok(()));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_close_ebadf() {
        assert_eq!(close(-1), Err(CloseError::Ebadf));
    }
}
