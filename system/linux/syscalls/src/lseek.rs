use celer_system_linux_ctypes::{OffT, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`lseek`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LseekError {
    /// `EBADF`.
    Ebadf,
    /// `EINVAL`.
    Einval,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by the underlying file object's seek operation.
    Other(Errno),
}

impl LseekError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Einval => Self::Einval,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Reposition the file offset for the open file descriptor `fd`.
///
/// This safe wrapper keeps the kernel-facing integer file descriptor, offset,
/// and `whence` selector, and maps the raw syscall return value into
/// `Result<OffT, LseekError>`.
///
/// On success, returns the resulting file offset.
///
/// See [`sys::lseek`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`LseekError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`LseekError::Einval`]: `whence` is unsupported or the requested file
///   position is invalid.
/// - [`LseekError::Eoverflow`]: the resulting offset does not fit in the raw
///   `off_t` return type.
/// - [`LseekError::Other`]: another error reported by the underlying file
///   object's seek implementation.
pub fn lseek(
    fd: UnsignedInt,
    offset: OffT,
    whence: UnsignedInt,
) -> Result<OffT, LseekError> {
    let ret = sys::lseek(fd, offset, whence);

    result_from_ret(ret as isize, |ret| ret as OffT, LseekError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::{Seek as _, Write as _},
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OffT, UnsignedInt};

    use crate::Errno;

    use super::{LseekError, lseek};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_lseek_{now}"));
        path
    }

    #[test]
    fn test_lseek_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(b"hello world").unwrap();
        file.rewind().unwrap();

        assert_eq!(
            lseek(file.as_raw_fd() as UnsignedInt, 0 as OffT, 1),
            Ok(0 as OffT)
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_lseek_invalid_fd() {
        assert_eq!(
            lseek(UnsignedInt::MAX, 0 as OffT, 0),
            Err(LseekError::Ebadf)
        );
    }

    #[test]
    fn test_lseek_invalid_whence() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        assert_eq!(
            lseek(file.as_raw_fd() as UnsignedInt, 0 as OffT, 99),
            Err(LseekError::Einval)
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_lseek_error_mapping() {
        assert_eq!(LseekError::from_errno(Errno::Ebadf), LseekError::Ebadf);
        assert_eq!(LseekError::from_errno(Errno::Einval), LseekError::Einval);
        assert_eq!(
            LseekError::from_errno(Errno::Eoverflow),
            LseekError::Eoverflow
        );
        assert_eq!(
            LseekError::from_errno(Errno::Enodev),
            LseekError::Other(Errno::Enodev)
        );
    }
}
