use core::{ffi::CStr, mem::MaybeUninit};

use celer_system_linux_ctypes::Stat;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`oldstat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldstatError {
    /// `EFAULT`.
    Efault,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated pathname lookup or filesystem code.
    Other(Errno),
}

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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{ffi::CString, mem::MaybeUninit};

    use crate::Errno;

    use super::{OldstatError, oldstat};

    #[test]
    fn test_oldstat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(oldstat(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[test]
    fn test_oldstat_missing_path() {
        let path = CString::new("/celer-oldstat-definitely-missing").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(
            oldstat(path.as_c_str(), &mut statbuf),
            Err(OldstatError::Other(Errno::Enoent))
        );
    }

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
}
