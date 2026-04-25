use core::ffi::CStr;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`umount`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UmountError {
    Eperm,
    Einval,
    Eacces,
    Enxio,
    Enoent,
    Ebusy,
    Enotdir,
    Other(Errno),
}

impl UmountError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::Eacces => Self::Eacces,
            Errno::Raw(6) => Self::Enxio,
            Errno::Enoent => Self::Enoent,
            Errno::Ebusy => Self::Ebusy,
            Errno::Enotdir => Self::Enotdir,
            errno => Self::Other(errno),
        }
    }
}

/// Unmount a filesystem or mount point.
///
/// This safe wrapper takes `name` as a NUL-terminated [`CStr`] pathname and
/// maps the raw return into `Result<(), UmountError>`. On x86_64 and aarch64,
/// the raw layer uses the `umount2` syscall slot with flags set to zero.
///
/// On success, the selected mount has been unmounted or, for the historical
/// root-filesystem case, remounted read-only.
///
/// See [`sys::umount`] for kernel behavior, required privileges, reachable
/// errors, and source references.
///
/// # Errors
/// - [`UmountError::Eperm`]: the caller lacks unmount permission.
/// - [`UmountError::Einval`]: the path does not identify a mount target.
/// - [`UmountError::Eacces`]: lookup or historical device access was denied.
/// - [`UmountError::Enxio`]: a historical block-device major was invalid.
/// - [`UmountError::Enoent`]: the path or mounted superblock was not found.
/// - [`UmountError::Ebusy`]: the target is still busy.
/// - [`UmountError::Enotdir`]: traversal needed a directory.
/// - [`UmountError::Other`]: delegated pathname or namespace unmount error.
pub fn umount(name: &CStr) -> Result<(), UmountError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::umount(name.as_ptr()) };
    unit_from_ret(ret as isize, UmountError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use super::{UmountError, umount};
    use crate::Errno;

    #[test]
    fn test_umount_missing_path_is_permission_or_lookup_error() {
        let path = CString::new("/celer-umount-definitely-missing").unwrap();
        let result = umount(path.as_c_str());

        assert!(matches!(
            result,
            Err(UmountError::Eperm | UmountError::Enoent)
        ));
    }

    #[test]
    fn test_umount_error_mapping() {
        assert_eq!(UmountError::from_errno(Errno::Eperm), UmountError::Eperm);
        assert_eq!(UmountError::from_errno(Errno::Einval), UmountError::Einval);
        assert_eq!(UmountError::from_errno(Errno::Eacces), UmountError::Eacces);
        assert_eq!(UmountError::from_errno(Errno::Raw(6)), UmountError::Enxio);
        assert_eq!(UmountError::from_errno(Errno::Enoent), UmountError::Enoent);
        assert_eq!(UmountError::from_errno(Errno::Ebusy), UmountError::Ebusy);
        assert_eq!(
            UmountError::from_errno(Errno::Enotdir),
            UmountError::Enotdir
        );
        assert_eq!(
            UmountError::from_errno(Errno::Eio),
            UmountError::Other(Errno::Eio)
        );
    }
}
