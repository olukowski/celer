use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`symlink`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SymlinkError {
    Efault,
    Enametoolong,
    Enoent,
    Enomem,
    Enotdir,
    Erofs,
    Eacces,
    Eperm,
    Other(Errno),
}

impl SymlinkError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enoent => Self::Enoent,
            Errno::Enomem => Self::Enomem,
            Errno::Enotdir => Self::Enotdir,
            Errno::Erofs => Self::Erofs,
            Errno::Eacces => Self::Eacces,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Create a symbolic link at `newname` whose stored target text is `oldname`.
///
/// This safe wrapper takes both raw pathname pointers as NUL-terminated
/// [`CStr`] values and maps the raw return into `Result<(), SymlinkError>`.
///
/// On success, the symlink was created. The kernel stores `oldname` as target
/// text; it does not resolve that path during creation.
///
/// See [`sys::symlink`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SymlinkError::Efault`]: the kernel could not read a pathname.
/// - [`SymlinkError::Enametoolong`]: a pathname is too long.
/// - [`SymlinkError::Enoent`]: a pathname is empty or a parent path is missing.
/// - [`SymlinkError::Enomem`]: pathname allocation failed.
/// - [`SymlinkError::Enotdir`]: traversal needed a directory.
/// - [`SymlinkError::Erofs`]: the destination is on a read-only filesystem.
/// - [`SymlinkError::Eacces`]: traversal or creation lacked permission.
/// - [`SymlinkError::Eperm`]: symlink creation is not permitted.
/// - [`SymlinkError::Other`]: delegated VFS, LSM, or filesystem error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn symlink(oldname: &CStr, newname: &CStr) -> Result<(), SymlinkError> {
    // SAFETY: both `CStr` values are readable NUL-terminated strings.
    let ret = unsafe { sys::symlink(oldname.as_ptr(), newname.as_ptr()) };
    unit_from_ret(ret as isize, SymlinkError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::{CString, OsStr},
        fs,
        os::unix::ffi::OsStrExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{SymlinkError, symlink};

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
    fn test_symlink_ok() {
        let link_path = temp_path("symlink_link");
        let target = CString::new("symlink_target").unwrap();
        let link_cstr =
            CString::new(link_path.as_os_str().as_encoded_bytes()).unwrap();

        symlink(target.as_c_str(), link_cstr.as_c_str()).unwrap();

        let stored = fs::read_link(&link_path).unwrap();
        assert_eq!(stored.as_os_str(), OsStr::from_bytes(b"symlink_target"));

        fs::remove_file(&link_path).unwrap();
    }

    #[test]
    fn test_symlink_existing_destination_is_other_eexist() {
        let link_path = temp_path("symlink_exists");
        fs::write(&link_path, b"occupied").unwrap();
        let target = CString::new("symlink_target").unwrap();
        let link_cstr =
            CString::new(link_path.as_os_str().as_encoded_bytes()).unwrap();

        let err = symlink(target.as_c_str(), link_cstr.as_c_str()).unwrap_err();
        assert_eq!(err, SymlinkError::Other(Errno::Eexist));

        fs::remove_file(&link_path).unwrap();
    }

    #[test]
    fn test_symlink_error_mapping() {
        assert_eq!(
            SymlinkError::from_errno(Errno::Efault),
            SymlinkError::Efault
        );
        assert_eq!(
            SymlinkError::from_errno(Errno::Raw(36)),
            SymlinkError::Enametoolong
        );
        assert_eq!(
            SymlinkError::from_errno(Errno::Enoent),
            SymlinkError::Enoent
        );
        assert_eq!(
            SymlinkError::from_errno(Errno::Enomem),
            SymlinkError::Enomem
        );
        assert_eq!(
            SymlinkError::from_errno(Errno::Enotdir),
            SymlinkError::Enotdir
        );
        assert_eq!(SymlinkError::from_errno(Errno::Erofs), SymlinkError::Erofs);
        assert_eq!(
            SymlinkError::from_errno(Errno::Eacces),
            SymlinkError::Eacces
        );
        assert_eq!(SymlinkError::from_errno(Errno::Eperm), SymlinkError::Eperm);
        assert_eq!(
            SymlinkError::from_errno(Errno::Eexist),
            SymlinkError::Other(Errno::Eexist)
        );
    }
}
