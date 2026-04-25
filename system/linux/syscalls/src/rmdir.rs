use core::ffi::CStr;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`rmdir`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RmdirError {
    Enoent,
    Enotempty,
    Einval,
    Ebusy,
    Eperm,
    Efault,
    Other(Errno),
}

impl RmdirError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enoent => Self::Enoent,
            Errno::Raw(39) => Self::Enotempty,
            Errno::Einval => Self::Einval,
            Errno::Ebusy => Self::Ebusy,
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Remove the empty directory named by `pathname`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<(), RmdirError>`.
///
/// See [`sys::rmdir`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`RmdirError::Enoent`]: the path is empty or missing.
/// - [`RmdirError::Enotempty`]: the target directory is not empty.
/// - [`RmdirError::Einval`]: the target is invalid for `rmdir`.
/// - [`RmdirError::Ebusy`]: the target is busy.
/// - [`RmdirError::Eperm`]: permission, sticky-bit, or filesystem rules
///   rejected the removal.
/// - [`RmdirError::Efault`]: the kernel could not read `pathname`.
/// - [`RmdirError::Other`]: another delegated pathname lookup, VFS,
///   filesystem, or security error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rmdir(pathname: &CStr) -> Result<(), RmdirError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::rmdir(pathname.as_ptr()) };
    unit_from_ret(ret as isize, RmdirError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{RmdirError, rmdir};

    fn temp_path(prefix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_{prefix}_{now}"));
        path
    }

    #[test]
    fn test_rmdir_ok() {
        let path = temp_path("rmdir_ok");
        fs::create_dir(&path).unwrap();
        let c_path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        rmdir(c_path.as_c_str()).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn test_rmdir_nonempty() {
        let path = temp_path("rmdir_nonempty");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("child"), b"child").unwrap();
        let c_path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(rmdir(c_path.as_c_str()), Err(RmdirError::Enotempty));

        fs::remove_file(path.join("child")).unwrap();
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn test_rmdir_error_mapping() {
        assert_eq!(RmdirError::from_errno(Errno::Enoent), RmdirError::Enoent);
        assert_eq!(
            RmdirError::from_errno(Errno::Raw(39)),
            RmdirError::Enotempty
        );
        assert_eq!(RmdirError::from_errno(Errno::Einval), RmdirError::Einval);
        assert_eq!(RmdirError::from_errno(Errno::Ebusy), RmdirError::Ebusy);
        assert_eq!(RmdirError::from_errno(Errno::Eperm), RmdirError::Eperm);
        assert_eq!(RmdirError::from_errno(Errno::Efault), RmdirError::Efault);
        assert_eq!(
            RmdirError::from_errno(Errno::Eio),
            RmdirError::Other(Errno::Eio)
        );
    }
}
