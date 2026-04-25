use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Stat, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`oldfstat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldfstatError {
    /// `EBADF`.
    Ebadf,
    /// `EOVERFLOW`.
    Eoverflow,
    /// `EFAULT`.
    Efault,
    /// Another errno returned by delegated file status code.
    Other(Errno),
}

impl OldfstatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Eoverflow => Self::Eoverflow,
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Get file status information through the legacy x86 `oldfstat` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Stat>` and maps the raw syscall return into
/// `Result<(), OldfstatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the open
/// file referenced by `fd`.
///
/// See [`sys::oldfstat`] for kernel behavior, reachable errors, ABI layout,
/// and source references.
///
/// # Errors
/// - [`OldfstatError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`OldfstatError::Eoverflow`]: metadata could not be represented in the
///   legacy stat layout.
/// - [`OldfstatError::Efault`]: the kernel could not write the output buffer.
/// - [`OldfstatError::Other`]: delegated file status error.
#[cfg(target_arch = "x86")]
pub fn oldfstat(
    fd: UnsignedInt,
    statbuf: &mut MaybeUninit<Stat>,
) -> Result<(), OldfstatError> {
    // SAFETY: `MaybeUninit<Stat>` provides writable storage for one legacy
    // stat value.
    let ret = unsafe { sys::oldfstat(fd, statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, OldfstatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs::File, mem::MaybeUninit, os::fd::AsRawFd as _};

    use crate::Errno;

    use super::{OldfstatError, oldfstat};

    #[test]
    fn test_oldfstat_ok() {
        let file = File::open("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(oldfstat(file.as_raw_fd() as u32, &mut statbuf), Ok(()));
    }

    #[test]
    fn test_oldfstat_invalid_fd() {
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(oldfstat(u32::MAX, &mut statbuf), Err(OldfstatError::Ebadf));
    }

    #[test]
    fn test_oldfstat_error_mapping() {
        assert_eq!(
            OldfstatError::from_errno(Errno::Ebadf),
            OldfstatError::Ebadf
        );
        assert_eq!(
            OldfstatError::from_errno(Errno::Eoverflow),
            OldfstatError::Eoverflow
        );
        assert_eq!(
            OldfstatError::from_errno(Errno::Efault),
            OldfstatError::Efault
        );
        assert_eq!(
            OldfstatError::from_errno(Errno::Eio),
            OldfstatError::Other(Errno::Eio)
        );
    }
}
