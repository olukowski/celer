use core::ffi::CStr;

use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`access`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AccessError {
    /// `EINVAL`.
    Einval,
    /// `ENOMEM`.
    Enomem,
    /// Another errno returned by delegated pathname, permission, or filesystem
    /// work.
    Other(Errno),
}

impl AccessError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Check whether the calling process can access a file by pathname.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<(), AccessError>`.
///
/// On success, returns `Ok(())` when the requested access is permitted.
///
/// See [`sys::access`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`AccessError::Einval`]: `mode` contains unsupported bits.
/// - [`AccessError::Enomem`]: temporary credential allocation failed.
/// - [`AccessError::Other`]: delegated pathname, permission, or filesystem
///   error.
pub fn access(pathname: &CStr, mode: Int) -> Result<(), AccessError> {
    // SAFETY: the `CStr` argument guarantees a valid, NUL-terminated
    // pathname pointer for the duration of the call.
    let ret = unsafe { sys::access(pathname.as_ptr(), mode) };

    unit_from_ret(ret as isize, AccessError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::Errno;

    use super::{AccessError, access};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_access_{now}"));
        path
    }

    #[test]
    fn test_access_ok() {
        let path = temp_path();
        File::create(&path).unwrap();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(access(path_c.as_c_str(), 0), Ok(()));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_access_einval() {
        let path = CString::new("/tmp").unwrap();

        assert_eq!(access(path.as_c_str(), 0x8000), Err(AccessError::Einval));
    }

    #[test]
    fn test_access_error_mapping() {
        assert_eq!(AccessError::from_errno(Errno::Einval), AccessError::Einval);
        assert_eq!(AccessError::from_errno(Errno::Enomem), AccessError::Enomem);
        assert_eq!(
            AccessError::from_errno(Errno::Enoent),
            AccessError::Other(Errno::Enoent)
        );
    }
}
