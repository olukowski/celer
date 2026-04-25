use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`unlink`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnlinkError {
    /// Another errno returned by pathname lookup, filesystem, or security
    /// handling.
    Other(Errno),
}

impl UnlinkError {
    fn from_errno(errno: Errno) -> Self {
        Self::Other(errno)
    }
}

/// Remove the directory entry named by `pathname`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return into `Result<(), UnlinkError>`.
///
/// On success, the named directory entry has been removed.
///
/// See [`sys::unlink`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`UnlinkError::Other`]: delegated pathname lookup, filesystem, or
///   security-hook error.
pub fn unlink(pathname: &CStr) -> Result<(), UnlinkError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::unlink(pathname.as_ptr()) };
    unit_from_ret(ret as isize, UnlinkError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::File,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{UnlinkError, unlink};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_unlink_{now}"));
        path
    }

    #[test]
    fn test_unlink_ok() {
        let path = temp_path();
        File::create(&path).unwrap();
        let c_path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(unlink(c_path.as_c_str()), Ok(()));
        assert!(!path.exists());
    }

    #[test]
    fn test_unlink_missing_path() {
        let path = CString::new("/definitely/not/a/real/celer-unlink").unwrap();

        assert_eq!(
            unlink(path.as_c_str()),
            Err(UnlinkError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_unlink_error_mapping() {
        assert_eq!(
            UnlinkError::from_errno(Errno::Eacces),
            UnlinkError::Other(Errno::Eacces)
        );
    }
}
