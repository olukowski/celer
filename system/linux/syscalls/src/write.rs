use celer_system_linux_ctypes::{Char, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`write`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// `EBADF`.
    Ebadf,
    /// `EFAULT`.
    Efault,
    /// `EINVAL`.
    Einval,
    /// Another errno returned by delegated file, driver, protocol, or security
    /// handling.
    Other(Errno),
}

impl WriteError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Write bytes from `buf` to `fd`.
///
/// This safe wrapper replaces the raw input pointer and byte count with a
/// shared byte slice and maps the raw syscall return into
/// `Result<usize, WriteError>`.
///
/// On success, returns the number of bytes written.
///
/// See [`sys::write`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`WriteError::Ebadf`]: `fd` is not open or is not open for writing.
/// - [`WriteError::Efault`]: the kernel could not read `buf`.
/// - [`WriteError::Einval`]: the target object cannot be written this way or
///   the request is invalid.
/// - [`WriteError::Other`]: delegated file, driver, protocol, or security
///   error.
pub fn write(fd: UnsignedInt, buf: &[u8]) -> Result<usize, WriteError> {
    // SAFETY: `buf` is readable for exactly `buf.len()` bytes.
    let ret = unsafe { sys::write(fd, buf.as_ptr().cast::<Char>(), buf.len()) };
    result_from_ret(ret as isize, |ret| ret as usize, WriteError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::OpenOptions,
        io::{Read as _, Seek as _},
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{WriteError, write};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_write_{now}"));
        path
    }

    #[test]
    fn test_write_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        assert_eq!(write(file.as_raw_fd() as u32, b"wrapped write"), Ok(13));

        file.rewind().unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"wrapped write");

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_write_invalid_fd() {
        assert_eq!(write(u32::MAX, b"x"), Err(WriteError::Ebadf));
    }

    #[test]
    fn test_write_error_mapping() {
        assert_eq!(WriteError::from_errno(Errno::Ebadf), WriteError::Ebadf);
        assert_eq!(WriteError::from_errno(Errno::Efault), WriteError::Efault);
        assert_eq!(WriteError::from_errno(Errno::Einval), WriteError::Einval);
        assert_eq!(
            WriteError::from_errno(Errno::Eio),
            WriteError::Other(Errno::Eio)
        );
    }
}
