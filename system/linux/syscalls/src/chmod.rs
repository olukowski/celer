use core::ffi::CStr;

use celer_system_linux_ctypes::UModeT;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`chmod`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChmodError {
    /// `EINTR`.
    Eintr,
    /// `EPERM`.
    Eperm,
    /// `EROFS`.
    Erofs,
    /// Another errno returned by delegated pathname, filesystem, or security
    /// hook work.
    Other(Errno),
}

impl ChmodError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eintr => Self::Eintr,
            Errno::Eperm => Self::Eperm,
            Errno::Erofs => Self::Erofs,
            errno => Self::Other(errno),
        }
    }
}

/// Change the mode bits of a file named by `pathname`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<(), ChmodError>`.
///
/// On success, returns `Ok(())` after the kernel applies the requested mode
/// bits, subject to normal chmod semantics such as clearing `S_ISGID` when the
/// caller is not allowed to keep it.
///
/// See [`sys::chmod`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ChmodError::Eintr`]: taking the inode lock was interrupted.
/// - [`ChmodError::Eperm`]: the caller was not allowed to change the mode.
/// - [`ChmodError::Erofs`]: the target mount is read-only.
/// - [`ChmodError::Other`]: delegated pathname, filesystem, or security-hook
///   error.
pub fn chmod(pathname: &CStr, mode: UModeT) -> Result<(), ChmodError> {
    // SAFETY: the `CStr` argument guarantees a valid, NUL-terminated pathname
    // pointer for the duration of the call.
    let ret = unsafe { sys::chmod(pathname.as_ptr(), mode) };

    unit_from_ret(ret as isize, ChmodError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, File},
        os::unix::fs::PermissionsExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UModeT;

    use crate::Errno;

    use super::{ChmodError, chmod};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_chmod_{now}"));
        path
    }

    #[test]
    fn test_chmod_ok() {
        let path = temp_path();
        File::create(&path).unwrap();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(chmod(path_c.as_c_str(), 0o600 as UModeT), Ok(()));

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_chmod_missing_path() {
        let path =
            CString::new("/definitely/not/a/real/celer-chmod-path").unwrap();

        assert_eq!(
            chmod(path.as_c_str(), 0o600 as UModeT),
            Err(ChmodError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_chmod_error_mapping() {
        assert_eq!(ChmodError::from_errno(Errno::Eintr), ChmodError::Eintr);
        assert_eq!(ChmodError::from_errno(Errno::Eperm), ChmodError::Eperm);
        assert_eq!(ChmodError::from_errno(Errno::Erofs), ChmodError::Erofs);
        assert_eq!(
            ChmodError::from_errno(Errno::Enoent),
            ChmodError::Other(Errno::Enoent)
        );
    }
}
