use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Char, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`read`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadError {
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

impl ReadError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Read bytes from `fd` into `buf`.
///
/// This safe wrapper replaces the raw output pointer and byte count with a
/// `&mut [MaybeUninit<u8>]` and maps the raw syscall return value into
/// `Result<usize, ReadError>`.
///
/// On success, returns the number of bytes initialized in `buf`.
///
/// See [`sys::read`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ReadError::Ebadf`]: `fd` is not open or is not open for reading.
/// - [`ReadError::Efault`]: the kernel could not write the output buffer.
/// - [`ReadError::Einval`]: the target object cannot be read this way or the
///   request is invalid.
/// - [`ReadError::Other`]: delegated file, driver, protocol, or security error.
pub fn read(
    fd: UnsignedInt,
    buf: &mut [MaybeUninit<u8>],
) -> Result<usize, ReadError> {
    // SAFETY: `MaybeUninit<u8>` has the same layout as `u8`/`Char`, and the
    // slice provides writable storage for exactly `buf.len()` bytes.
    let ret =
        unsafe { sys::read(fd, buf.as_mut_ptr().cast::<Char>(), buf.len()) };

    result_from_ret(ret as isize, |ret| ret as usize, ReadError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::File,
        io::Write as _,
        mem::MaybeUninit,
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{ReadError, read};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_read_{now}"));
        path
    }

    #[test]
    fn test_read_ok() {
        let path = temp_path();
        let mut file = File::create(&path).unwrap();
        file.write_all(b"read bytes").unwrap();
        drop(file);

        let file = File::open(&path).unwrap();
        let mut buf = [MaybeUninit::uninit(); 16];

        let n = read(file.as_raw_fd() as u32, &mut buf).unwrap();
        assert_eq!(n, 10);
        let initialized = unsafe {
            core::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), n)
        };
        assert_eq!(initialized, b"read bytes");

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_read_invalid_fd() {
        let mut buf = [MaybeUninit::uninit(); 1];

        assert_eq!(read(u32::MAX, &mut buf), Err(ReadError::Ebadf));
    }

    #[test]
    fn test_read_error_mapping() {
        assert_eq!(ReadError::from_errno(Errno::Ebadf), ReadError::Ebadf);
        assert_eq!(ReadError::from_errno(Errno::Efault), ReadError::Efault);
        assert_eq!(ReadError::from_errno(Errno::Einval), ReadError::Einval);
        assert_eq!(
            ReadError::from_errno(Errno::Eio),
            ReadError::Other(Errno::Eio)
        );
    }
}
