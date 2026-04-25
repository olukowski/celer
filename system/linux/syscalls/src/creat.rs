use core::ffi::CStr;

use celer_system_linux_ctypes::{Int, UModeT};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`creat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreatError {
    /// Another errno returned by delegated pathname lookup, file creation, or
    /// file-descriptor allocation work.
    Other(Errno),
}

impl CreatError {
    fn from_errno(errno: Errno) -> Self {
        Self::Other(errno)
    }
}

/// Create or truncate `pathname` and return the opened file descriptor.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<Int, CreatError>`.
///
/// On success, returns the nonnegative file descriptor opened with the raw
/// [`sys::creat`] semantics.
///
/// See [`sys::creat`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`CreatError::Other`]: delegated pathname lookup, file creation, or
///   file-descriptor allocation error.
pub fn creat(pathname: &CStr, mode: UModeT) -> Result<Int, CreatError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pathname pointer for
    // the duration of the call.
    let ret = unsafe { sys::creat(pathname.as_ptr(), mode) };

    result_from_ret(ret as isize, |ret| ret as Int, CreatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs,
        os::fd::FromRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UModeT;

    use super::{CreatError, creat};
    use crate::Errno;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_creat_{now}"));
        path
    }

    #[test]
    fn test_creat_ok() {
        let path = temp_path();
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        let fd = creat(path_cstr.as_c_str(), 0o640 as UModeT)
            .expect("creat should succeed for a temp path");

        let file = unsafe { std::fs::File::from_raw_fd(fd) };
        drop(file);

        let metadata =
            fs::metadata(&path).expect("creat should create the file");
        assert!(metadata.is_file(), "created path should be a regular file");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_creat_missing_parent() {
        let mut path = temp_path();
        path.push("missing-parent");
        path.push("file");
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            creat(path_cstr.as_c_str(), 0o640 as UModeT),
            Err(CreatError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_creat_error_mapping() {
        assert_eq!(
            CreatError::from_errno(Errno::Enoent),
            CreatError::Other(Errno::Enoent)
        );
    }
}
