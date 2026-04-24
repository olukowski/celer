use core::ffi::CStr;

use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::sys;

/// Errors returned by [`access`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccessError {
    Einval,
    Enomem,
    Other(Errno),
}

impl From<isize> for AccessError {
    fn from(value: isize) -> Self {
        match Errno::from(value) {
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Check whether the calling process can access a file by pathname.
///
/// On success, returns `Ok(())` when the requested access is permitted.
///
/// # Errors
/// - `EINVAL`: `mode` contains unsupported bits.
/// - `ENOMEM`: temporary override credential allocation failed.
/// - `Other(..)`: pathname resolution, inode permissions, or filesystem-specific
///   checks returned a delegated errno.
pub fn access(pathname: &CStr, mode: Int) -> Result<(), AccessError> {
    // SAFETY: the `CStr` argument guarantees a valid, NUL-terminated
    // pathname pointer for the duration of the call.
    let ret = unsafe { sys::access(pathname.as_ptr(), mode) };

    if Errno::is_errno(ret) {
        Err(AccessError::from(ret))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;
    use std::fs::File;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        let path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(access(path.as_c_str(), 0), Ok(()));
    }

    #[test]
    fn test_access_einval() {
        let path = CString::new("/tmp").unwrap();

        assert_eq!(access(path.as_c_str(), 0x8000), Err(AccessError::Einval));
    }
}
