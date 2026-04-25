use core::ffi::CStr;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`rename`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenameError {
    Enoent,
    Eexist,
    Exdev,
    Eperm,
    Efault,
    Other(Errno),
}

impl RenameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enoent => Self::Enoent,
            Errno::Eexist => Self::Eexist,
            Errno::Exdev => Self::Exdev,
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Rename the filesystem object named by `oldname` to `newname`.
///
/// This safe wrapper takes two NUL-terminated [`CStr`] pathnames and maps the
/// raw syscall return value into `Result<(), RenameError>`.
///
/// See [`sys::rename`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`RenameError::Enoent`]: a required path component or source object is
///   missing.
/// - [`RenameError::Eexist`]: the destination already exists in a path where
///   replacement is not allowed.
/// - [`RenameError::Exdev`]: the rename crosses mount points.
/// - [`RenameError::Eperm`]: permission, sticky-bit, or filesystem rules
///   rejected the rename.
/// - [`RenameError::Efault`]: the kernel could not read a pathname.
/// - [`RenameError::Other`]: another delegated pathname lookup, VFS,
///   filesystem, or security error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rename(oldname: &CStr, newname: &CStr) -> Result<(), RenameError> {
    // SAFETY: `CStr` values provide readable NUL-terminated pathnames.
    let ret = unsafe { sys::rename(oldname.as_ptr(), newname.as_ptr()) };
    unit_from_ret(ret as isize, RenameError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, File},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{RenameError, rename};

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
    fn test_rename_ok() {
        let old_path = temp_path("rename_old");
        let new_path = temp_path("rename_new");
        File::create(&old_path).unwrap();
        let old =
            CString::new(old_path.as_os_str().as_encoded_bytes()).unwrap();
        let new =
            CString::new(new_path.as_os_str().as_encoded_bytes()).unwrap();

        rename(old.as_c_str(), new.as_c_str()).unwrap();

        assert!(!old_path.exists());
        assert!(new_path.exists());
        fs::remove_file(new_path).unwrap();
    }

    #[test]
    fn test_rename_missing_source() {
        let old_path = temp_path("rename_missing_old");
        let new_path = temp_path("rename_missing_new");
        let old =
            CString::new(old_path.as_os_str().as_encoded_bytes()).unwrap();
        let new =
            CString::new(new_path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            rename(old.as_c_str(), new.as_c_str()),
            Err(RenameError::Enoent)
        );
    }

    #[test]
    fn test_rename_error_mapping() {
        assert_eq!(RenameError::from_errno(Errno::Enoent), RenameError::Enoent);
        assert_eq!(RenameError::from_errno(Errno::Eexist), RenameError::Eexist);
        assert_eq!(RenameError::from_errno(Errno::Exdev), RenameError::Exdev);
        assert_eq!(RenameError::from_errno(Errno::Eperm), RenameError::Eperm);
        assert_eq!(RenameError::from_errno(Errno::Efault), RenameError::Efault);
        assert_eq!(
            RenameError::from_errno(Errno::Eio),
            RenameError::Other(Errno::Eio)
        );
    }
}
