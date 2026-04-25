use core::{ffi::CStr, mem::MaybeUninit};

use celer_system_linux_ctypes::Stat;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`oldlstat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldlstatError {
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

impl OldlstatError {
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

/// Get file status information through the legacy x86 `oldlstat` ABI.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer with `&mut MaybeUninit<Stat>`, and maps the raw syscall
/// return into `Result<(), OldlstatError>`.
///
/// On success, the kernel has initialized `statbuf` with metadata for the
/// resolved path without following a final symlink.
///
/// See [`sys::oldlstat`] for kernel behavior, reachable errors, ABI layout,
/// and source references.
///
/// # Errors
/// - [`OldlstatError::Efault`]: the kernel could not read the path or write
///   the output buffer.
/// - [`OldlstatError::Enametoolong`]: the pathname is too long.
/// - [`OldlstatError::Enoent`]: the path is empty or missing.
/// - [`OldlstatError::Enomem`]: the kernel could not allocate pathname storage.
/// - [`OldlstatError::Enotdir`]: traversal needed a directory.
/// - [`OldlstatError::Eacces`]: traversal lacked search permission.
/// - [`OldlstatError::Eoverflow`]: metadata could not be represented in the
///   legacy stat layout.
/// - [`OldlstatError::Other`]: delegated pathname lookup or filesystem error.
#[cfg(target_arch = "x86")]
pub fn oldlstat(
    filename: &CStr,
    statbuf: &mut MaybeUninit<Stat>,
) -> Result<(), OldlstatError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<Stat>` provides writable storage for one legacy stat value.
    let ret = unsafe { sys::oldlstat(filename.as_ptr(), statbuf.as_mut_ptr()) };

    unit_from_ret(ret as isize, OldlstatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{ffi::CString, mem::MaybeUninit};

    use crate::Errno;

    use super::{OldlstatError, oldlstat};

    #[test]
    fn test_oldlstat_ok() {
        let path = CString::new("/").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(oldlstat(path.as_c_str(), &mut statbuf), Ok(()));
    }

    #[test]
    fn test_oldlstat_missing_path() {
        let path = CString::new("/celer-oldlstat-definitely-missing").unwrap();
        let mut statbuf = MaybeUninit::uninit();

        assert_eq!(
            oldlstat(path.as_c_str(), &mut statbuf),
            Err(OldlstatError::Enoent)
        );
    }

    #[test]
    fn test_oldlstat_error_mapping() {
        assert_eq!(
            OldlstatError::from_errno(Errno::Efault),
            OldlstatError::Efault
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Raw(36)),
            OldlstatError::Enametoolong
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Enoent),
            OldlstatError::Enoent
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Enomem),
            OldlstatError::Enomem
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Enotdir),
            OldlstatError::Enotdir
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Eacces),
            OldlstatError::Eacces
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Eoverflow),
            OldlstatError::Eoverflow
        );
        assert_eq!(
            OldlstatError::from_errno(Errno::Eio),
            OldlstatError::Other(Errno::Eio)
        );
    }
}
