use core::ffi::CStr;

use celer_system_linux_ctypes::UModeT;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`mkdir`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MkdirError {
    /// Another errno returned by delegated pathname lookup, directory create
    /// operations, or security hooks.
    Other(Errno),
}

impl MkdirError {
    fn from_errno(errno: Errno) -> Self {
        Self::Other(errno)
    }
}

/// Create a directory named by `pathname`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<(), MkdirError>`.
///
/// On success, returns `Ok(())` after the kernel has created the directory.
///
/// See [`sys::mkdir`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`MkdirError::Other`]: delegated pathname lookup, directory create, or
///   security-hook error.
pub fn mkdir(pathname: &CStr, mode: UModeT) -> Result<(), MkdirError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pathname pointer for
    // the duration of the call.
    let ret = unsafe { sys::mkdir(pathname.as_ptr(), mode) };

    unit_from_ret(ret as isize, MkdirError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UModeT;

    use crate::Errno;

    use super::{MkdirError, mkdir};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_mkdir_{now}"));
        path
    }

    #[test]
    fn test_mkdir_ok() {
        let path = temp_path();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(mkdir(path_c.as_c_str(), 0o700 as UModeT), Ok(()));

        assert!(fs::metadata(&path).unwrap().is_dir());
        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_mkdir_existing_path() {
        let path = temp_path();
        fs::create_dir(&path).unwrap();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            mkdir(path_c.as_c_str(), 0o700 as UModeT),
            Err(MkdirError::Other(Errno::Eexist))
        );

        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_mkdir_error_mapping() {
        assert_eq!(
            MkdirError::from_errno(Errno::Enoent),
            MkdirError::Other(Errno::Enoent)
        );
    }
}
