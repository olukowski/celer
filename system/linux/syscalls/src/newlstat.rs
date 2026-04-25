use core::{ffi::CStr, mem::MaybeUninit};

#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::NewStat as NativeStat;
#[cfg(target_arch = "x86_64")]
use celer_system_linux_ctypes::Stat64 as NativeStat;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::NewStat as Linux10NewStat;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`newlstat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewlstatError {
    /// `EFAULT`.
    Efault,
    /// `ENAMETOOLONG`.
    Enametoolong,
    /// `ENOENT`.
    Enoent,
    /// `ENOMEM`.
    Enomem,
    /// `ENOTDIR`.
    Enotdir,
    /// `EACCES`.
    Eacces,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated pathname lookup or filesystem code.
    Other(Errno),
}

impl NewlstatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enoent => Self::Enoent,
            Errno::Enomem => Self::Enomem,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eacces => Self::Eacces,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Get file status information for a path without following the final symlink.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer with `&mut MaybeUninit<NativeStat>`, and maps the raw syscall
/// return into `Result<(), NewlstatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the
/// resolved path.
///
/// See [`sys::newlstat`] for kernel behavior, reachable errors, ABI layouts,
/// and source references.
///
/// # Errors
/// - [`NewlstatError::Efault`]: the kernel could not read the path or write
///   the output buffer.
/// - [`NewlstatError::Enametoolong`]: the pathname is too long.
/// - [`NewlstatError::Enoent`]: the path is empty or missing.
/// - [`NewlstatError::Enomem`]: the kernel could not allocate pathname storage.
/// - [`NewlstatError::Enotdir`]: traversal needed a directory.
/// - [`NewlstatError::Eacces`]: traversal lacked search permission.
/// - [`NewlstatError::Eoverflow`]: metadata could not be represented in the
///   native stat layout.
/// - [`NewlstatError::Other`]: delegated pathname lookup or filesystem error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn newlstat(
    filename: &CStr,
    statbuf: &mut MaybeUninit<NativeStat>,
) -> Result<(), NewlstatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<NativeStat>` provides writable storage for one stat value.
    let ret = unsafe { sys::newlstat(filename.as_ptr(), statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, NewlstatError::from_errno)
}

/// Get file status information through the Linux 1.0 `sys_newlstat` ABI.
///
/// This safe wrapper mirrors [`newlstat`], but uses
/// `&mut MaybeUninit<Linux10NewStat>` for the Linux 1.0 layout exposed from
/// [`crate::linux_1_0`].
///
/// See [`sys::linux_1_0::newlstat`] for historical kernel behavior and source
/// references.
#[cfg(target_arch = "x86")]
pub fn newlstat_1_0(
    filename: &CStr,
    statbuf: &mut MaybeUninit<Linux10NewStat>,
) -> Result<(), NewlstatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<Linux10NewStat>` provides writable storage for one Linux
    // 1.0 stat value.
    let ret = unsafe {
        sys::linux_1_0::newlstat(filename.as_ptr(), statbuf.as_mut_ptr())
    };

    unit_from_ret(ret as isize, NewlstatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{ffi::CString, mem::MaybeUninit};

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::newlstat_1_0;
    use super::{NewlstatError, newlstat};

    #[test]
    fn test_newlstat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(newlstat(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_newlstat_1_0_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(newlstat_1_0(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[test]
    fn test_newlstat_missing_path() {
        let path = CString::new("/celer-newlstat-definitely-missing").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(
            newlstat(path.as_c_str(), &mut statbuf),
            Err(NewlstatError::Enoent)
        );
    }

    #[test]
    fn test_newlstat_error_mapping() {
        assert_eq!(
            NewlstatError::from_errno(Errno::Efault),
            NewlstatError::Efault
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Raw(36)),
            NewlstatError::Enametoolong
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Enoent),
            NewlstatError::Enoent
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Enomem),
            NewlstatError::Enomem
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Enotdir),
            NewlstatError::Enotdir
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Eacces),
            NewlstatError::Eacces
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Eoverflow),
            NewlstatError::Eoverflow
        );
        assert_eq!(
            NewlstatError::from_errno(Errno::Eio),
            NewlstatError::Other(Errno::Eio)
        );
    }
}
