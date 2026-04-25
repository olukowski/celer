use core::{ffi::CStr, mem::MaybeUninit};

#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::NewStat as NativeStat;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::Stat;
#[cfg(target_arch = "x86_64")]
use celer_system_linux_ctypes::Stat64 as NativeStat;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::NewStat as Linux10NewStat;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`oldstat`].
#[cfg(target_arch = "x86")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldstatError {
    /// `EFAULT`.
    Efault,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated pathname lookup or filesystem code.
    Other(Errno),
}

#[cfg(target_arch = "x86")]
impl OldstatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Get file status information through the legacy x86 `oldstat` ABI.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer with `&mut MaybeUninit<Stat>`, and maps the raw syscall
/// return into `Result<(), OldstatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the
/// resolved path.
///
/// See [`sys::oldstat`] for kernel behavior, reachable errors, ABI layout, and
/// source references.
///
/// # Errors
/// - [`OldstatError::Efault`]: the kernel could not read the path or write the
///   output buffer.
/// - [`OldstatError::Eoverflow`]: metadata could not be represented in the
///   legacy stat layout.
/// - [`OldstatError::Other`]: delegated pathname lookup or filesystem error.
#[cfg(target_arch = "x86")]
pub fn oldstat(
    filename: &CStr,
    statbuf: &mut MaybeUninit<Stat>,
) -> Result<(), OldstatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<Stat>` provides writable storage for one legacy stat value.
    let ret = unsafe { sys::oldstat(filename.as_ptr(), statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, OldstatError::from_errno)
}

/// Errors returned by [`stat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatError {
    /// `EFAULT`.
    Efault,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated pathname lookup or filesystem code.
    Other(Errno),
}

impl StatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Get file status information for the path named by `filename`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer with `&mut MaybeUninit<NativeStat>`, and maps the raw syscall
/// return into `Result<(), StatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the
/// resolved path using the target architecture's native stat layout.
///
/// See [`sys::stat`] for kernel behavior, reachable errors, ABI layout, and
/// source references.
///
/// # Errors
/// - [`StatError::Efault`]: the kernel could not read the path or write the
///   output buffer.
/// - [`StatError::Eoverflow`]: metadata could not be represented in the native
///   stat layout.
/// - [`StatError::Other`]: delegated pathname lookup or filesystem error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn stat(
    filename: &CStr,
    statbuf: &mut MaybeUninit<NativeStat>,
) -> Result<(), StatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<NativeStat>` provides writable storage for one native stat
    // value.
    let ret = unsafe { sys::stat(filename.as_ptr(), statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, StatError::from_errno)
}

/// Get file status information through the Linux 1.0 `sys_newstat` ABI.
///
/// This safe wrapper mirrors [`stat`], but uses the Linux 1.0 newstat layout
/// exposed from [`crate::linux_1_0`].
///
/// See [`sys::linux_1_0::stat`] for historical kernel behavior and source
/// references.
#[cfg(target_arch = "x86")]
pub fn stat_1_0(
    filename: &CStr,
    statbuf: &mut MaybeUninit<Linux10NewStat>,
) -> Result<(), StatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<Linux10NewStat>` provides writable storage for one Linux
    // 1.0 newstat value.
    let ret = unsafe {
        sys::linux_1_0::stat(filename.as_ptr(), statbuf.as_mut_ptr())
    };

    unit_from_ret(ret as isize, StatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{ffi::CString, mem::MaybeUninit};

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::stat_1_0;
    #[cfg(target_arch = "x86")]
    use super::{OldstatError, oldstat};
    use super::{StatError, stat};

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldstat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(oldstat(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldstat_missing_path() {
        let path = CString::new("/celer-oldstat-definitely-missing").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(
            oldstat(path.as_c_str(), &mut statbuf),
            Err(OldstatError::Other(Errno::Enoent))
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldstat_error_mapping() {
        assert_eq!(
            OldstatError::from_errno(Errno::Efault),
            OldstatError::Efault
        );
        assert_eq!(
            OldstatError::from_errno(Errno::Eoverflow),
            OldstatError::Eoverflow
        );
        assert_eq!(
            OldstatError::from_errno(Errno::Enoent),
            OldstatError::Other(Errno::Enoent)
        );
    }

    #[test]
    fn test_stat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(stat(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_stat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(stat_1_0(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[test]
    fn test_stat_missing_path() {
        let path = CString::new("/celer-stat-definitely-missing").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(
            stat(path.as_c_str(), &mut statbuf),
            Err(StatError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_stat_error_mapping() {
        assert_eq!(StatError::from_errno(Errno::Efault), StatError::Efault);
        assert_eq!(
            StatError::from_errno(Errno::Eoverflow),
            StatError::Eoverflow
        );
        assert_eq!(
            StatError::from_errno(Errno::Enoent),
            StatError::Other(Errno::Enoent)
        );
    }
}
