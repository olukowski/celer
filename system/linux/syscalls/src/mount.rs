use core::ffi::CStr;

use celer_system_linux_ctypes::{UnsignedLong, Void};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`mount`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MountError {
    /// `EPERM`.
    Eperm,
    /// `EFAULT`.
    Efault,
    /// `ENOMEM`.
    Enomem,
    /// `ENODEV`.
    Enodev,
    /// `ENOTBLK`.
    Enotblk,
    /// `EACCES`.
    Eacces,
    /// `ENXIO`.
    Enxio,
    /// `EMFILE`.
    Emfile,
    /// `EBUSY`.
    Ebusy,
    /// `EINVAL`.
    Einval,
    /// Another errno returned by pathname lookup, block-driver, filesystem, or
    /// security-hook mount code.
    Other(Errno),
}

impl MountError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            Errno::Enomem => Self::Enomem,
            Errno::Enodev => Self::Enodev,
            Errno::Raw(15) => Self::Enotblk,
            Errno::Eacces => Self::Eacces,
            Errno::Raw(6) => Self::Enxio,
            Errno::Emfile => Self::Emfile,
            Errno::Ebusy => Self::Ebusy,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Mount or remount a filesystem.
///
/// This wrapper takes `target` as a required NUL-terminated [`CStr`], accepts
/// nullable `source` and `filesystemtype` strings as `Option<&CStr>`, keeps the
/// raw `mountflags` and filesystem-specific `data` pointer, and maps the raw
/// return into `Result<(), MountError>`.
///
/// On success, the requested mount operation has completed.
///
/// See [`sys::mount`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// The caller must ensure `data`, when non-null, satisfies the memory contract
/// required by the selected filesystem and mount flags for the duration of the
/// syscall.
///
/// # Errors
/// - [`MountError::Eperm`]: the caller lacks permission, or historical kernels
///   rejected the target type.
/// - [`MountError::Efault`]: a copied string or data pointer was inaccessible.
/// - [`MountError::Enomem`]: the kernel could not allocate temporary storage.
/// - [`MountError::Enodev`]: the filesystem type is unavailable.
/// - [`MountError::Enotblk`]: a block-device-backed filesystem got a
///   non-block source.
/// - [`MountError::Eacces`]: access to the source or traversal was denied.
/// - [`MountError::Enxio`]: the source block device number is invalid.
/// - [`MountError::Emfile`]: no unnamed device number was available.
/// - [`MountError::Ebusy`]: the target or source was busy.
/// - [`MountError::Einval`]: remount validation failed.
/// - [`MountError::Other`]: delegated pathname lookup, block-driver,
///   filesystem, or security-hook mount error.
pub unsafe fn mount(
    source: Option<&CStr>,
    target: &CStr,
    filesystemtype: Option<&CStr>,
    mountflags: UnsignedLong,
    data: *const Void,
) -> Result<(), MountError> {
    // SAFETY: `CStr` values are valid NUL-terminated strings, null is passed
    // only for optional strings, and the caller upholds `data`'s contract.
    let ret = unsafe {
        sys::mount(
            source.map_or(core::ptr::null(), CStr::as_ptr),
            target.as_ptr(),
            filesystemtype.map_or(core::ptr::null(), CStr::as_ptr),
            mountflags,
            data,
        )
    };

    unit_from_ret(ret as isize, MountError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use crate::Errno;

    use super::{MountError, mount};

    #[test]
    fn test_mount_requires_permission_or_valid_target() {
        let target = CString::new("/").unwrap();
        let fstype = CString::new("definitely-not-a-celer-fs").unwrap();

        let err = unsafe {
            mount(
                None,
                target.as_c_str(),
                Some(fstype.as_c_str()),
                0,
                core::ptr::null(),
            )
        }
        .unwrap_err();

        assert!(matches!(err, MountError::Eperm | MountError::Enodev));
    }

    #[test]
    fn test_mount_error_mapping() {
        assert_eq!(MountError::from_errno(Errno::Eperm), MountError::Eperm);
        assert_eq!(MountError::from_errno(Errno::Efault), MountError::Efault);
        assert_eq!(MountError::from_errno(Errno::Enomem), MountError::Enomem);
        assert_eq!(MountError::from_errno(Errno::Enodev), MountError::Enodev);
        assert_eq!(MountError::from_errno(Errno::Raw(15)), MountError::Enotblk);
        assert_eq!(MountError::from_errno(Errno::Eacces), MountError::Eacces);
        assert_eq!(MountError::from_errno(Errno::Raw(6)), MountError::Enxio);
        assert_eq!(MountError::from_errno(Errno::Emfile), MountError::Emfile);
        assert_eq!(MountError::from_errno(Errno::Ebusy), MountError::Ebusy);
        assert_eq!(MountError::from_errno(Errno::Einval), MountError::Einval);
        assert_eq!(
            MountError::from_errno(Errno::Enoent),
            MountError::Other(Errno::Enoent)
        );
    }
}
