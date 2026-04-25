use core::mem::MaybeUninit;

#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::NewStat as NativeStat;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
use celer_system_linux_ctypes::Stat64 as NativeStat;
use celer_system_linux_ctypes::UnsignedInt;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::NewStat as Linux10NewStat;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`newfstat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewfstatError {
    /// `EFAULT`.
    Efault,
    /// `EBADF`.
    Ebadf,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated file status code.
    Other(Errno),
}

impl NewfstatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Ebadf => Self::Ebadf,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Get file status information for an open file descriptor.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<NativeStat>` and maps the raw syscall return into
/// `Result<(), NewfstatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the open
/// file referenced by `fd`.
///
/// See [`sys::newfstat`] for kernel behavior, reachable errors, ABI layouts,
/// and source references.
///
/// # Errors
/// - [`NewfstatError::Efault`]: the kernel could not write the output buffer.
/// - [`NewfstatError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`NewfstatError::Eoverflow`]: metadata could not be represented in the
///   native stat layout.
/// - [`NewfstatError::Other`]: delegated file status error.
pub fn newfstat(
    fd: UnsignedInt,
    statbuf: &mut MaybeUninit<NativeStat>,
) -> Result<(), NewfstatError> {
    // SAFETY: `MaybeUninit<NativeStat>` provides writable storage for one
    // native stat value.
    let ret = unsafe { sys::newfstat(fd, statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, NewfstatError::from_errno)
}

/// Get file status information through the Linux 1.0 `sys_newfstat` ABI.
///
/// This safe wrapper mirrors [`newfstat`], but uses
/// `&mut MaybeUninit<Linux10NewStat>` for the Linux 1.0 layout exposed from
/// [`crate::linux_1_0`].
///
/// See [`sys::linux_1_0::newfstat`] for historical kernel behavior and source
/// references.
#[cfg(target_arch = "x86")]
pub fn newfstat_1_0(
    fd: UnsignedInt,
    statbuf: &mut MaybeUninit<Linux10NewStat>,
) -> Result<(), NewfstatError> {
    // SAFETY: `MaybeUninit<Linux10NewStat>` provides writable storage for one
    // Linux 1.0 stat value.
    let ret = unsafe { sys::linux_1_0::newfstat(fd, statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, NewfstatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs::File, mem::MaybeUninit, os::fd::AsRawFd as _};

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::newfstat_1_0;
    use super::{NewfstatError, newfstat};

    #[test]
    fn test_newfstat_ok() {
        let file = File::open("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(newfstat(file.as_raw_fd() as u32, &mut statbuf), Ok(()));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_newfstat_1_0_ok() {
        let file = File::open("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(newfstat_1_0(file.as_raw_fd() as u32, &mut statbuf), Ok(()));
    }

    #[test]
    fn test_newfstat_invalid_fd() {
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(newfstat(u32::MAX, &mut statbuf), Err(NewfstatError::Ebadf));
    }

    #[test]
    fn test_newfstat_error_mapping() {
        assert_eq!(
            NewfstatError::from_errno(Errno::Efault),
            NewfstatError::Efault
        );
        assert_eq!(
            NewfstatError::from_errno(Errno::Ebadf),
            NewfstatError::Ebadf
        );
        assert_eq!(
            NewfstatError::from_errno(Errno::Eoverflow),
            NewfstatError::Eoverflow
        );
        assert_eq!(
            NewfstatError::from_errno(Errno::Eio),
            NewfstatError::Other(Errno::Eio)
        );
    }
}
