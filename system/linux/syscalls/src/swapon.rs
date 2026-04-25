use core::ffi::CStr;

use celer_system_linux_ctypes::Int;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`swapon`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SwaponError {
    Eperm,
    Efault,
    Einval,
    Enomem,
    Enoent,
    Enametoolong,
    Enotdir,
    Eacces,
    Eloop,
    Ebusy,
    Enodev,
    Other(Errno),
}

impl SwaponError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Enoent => Self::Enoent,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eacces => Self::Eacces,
            Errno::Eloop => Self::Eloop,
            Errno::Ebusy => Self::Ebusy,
            Errno::Enodev => Self::Enodev,
            errno => Self::Other(errno),
        }
    }
}

/// Enable swapping on a block device or regular file.
///
/// This safe wrapper takes `specialfile` as a NUL-terminated [`CStr`] pathname,
/// keeps the raw `swap_flags` integer argument, and maps the raw return into
/// `Result<(), SwaponError>`.
///
/// On success, the kernel has activated the selected swap area.
///
/// See [`sys::swapon`] for kernel behavior, required privileges, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SwaponError::Eperm`]: the caller lacks permission or no swap slot is
///   available on historical kernels.
/// - [`SwaponError::Efault`]: the kernel could not read `specialfile`.
/// - [`SwaponError::Einval`]: unsupported `swap_flags`, unsupported target
///   type, invalid swap signature, or no usable swap pages.
/// - [`SwaponError::Enomem`]: pathname or swap bookkeeping allocation failed.
/// - [`SwaponError::Enoent`]: the path is empty or missing.
/// - [`SwaponError::Enametoolong`]: the pathname is too long.
/// - [`SwaponError::Enotdir`]: traversal needed a directory.
/// - [`SwaponError::Eacces`]: traversal lacked search permission.
/// - [`SwaponError::Eloop`]: pathname resolution encountered a symlink loop.
/// - [`SwaponError::Ebusy`]: the target is busy or already active as swap.
/// - [`SwaponError::Enodev`]: a historical block-device target had no device.
/// - [`SwaponError::Other`]: delegated pathname, file-open, or swap setup
///   error.
pub fn swapon(specialfile: &CStr, swap_flags: Int) -> Result<(), SwaponError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::swapon(specialfile.as_ptr(), swap_flags) };
    unit_from_ret(ret as isize, SwaponError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use super::{SwaponError, swapon};
    use crate::Errno;

    #[test]
    fn test_swapon_missing_path_is_permission_or_lookup_error() {
        let path = CString::new("/celer-swapon-definitely-missing").unwrap();
        let result = swapon(path.as_c_str(), 0);

        assert!(matches!(
            result,
            Err(SwaponError::Eperm | SwaponError::Enoent)
        ));
    }

    #[test]
    fn test_swapon_error_mapping() {
        assert_eq!(SwaponError::from_errno(Errno::Eperm), SwaponError::Eperm);
        assert_eq!(SwaponError::from_errno(Errno::Efault), SwaponError::Efault);
        assert_eq!(SwaponError::from_errno(Errno::Einval), SwaponError::Einval);
        assert_eq!(SwaponError::from_errno(Errno::Enomem), SwaponError::Enomem);
        assert_eq!(SwaponError::from_errno(Errno::Enoent), SwaponError::Enoent);
        assert_eq!(
            SwaponError::from_errno(Errno::Raw(36)),
            SwaponError::Enametoolong
        );
        assert_eq!(
            SwaponError::from_errno(Errno::Enotdir),
            SwaponError::Enotdir
        );
        assert_eq!(SwaponError::from_errno(Errno::Eacces), SwaponError::Eacces);
        assert_eq!(SwaponError::from_errno(Errno::Eloop), SwaponError::Eloop);
        assert_eq!(SwaponError::from_errno(Errno::Ebusy), SwaponError::Ebusy);
        assert_eq!(SwaponError::from_errno(Errno::Enodev), SwaponError::Enodev);
        assert_eq!(
            SwaponError::from_errno(Errno::Eio),
            SwaponError::Other(Errno::Eio)
        );
    }
}
