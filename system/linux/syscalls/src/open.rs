use core::ffi::CStr;

use celer_system_linux_ctypes::{Int, UModeT};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`open`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// `EINVAL`.
    Einval,
    /// Another errno returned by delegated pathname lookup, file open,
    /// permission, filesystem, security-hook, or descriptor-allocation code.
    Other(Errno),
}

impl OpenError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Open `filename` with the given raw `flags` and `mode`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<Int, OpenError>`.
///
/// On success, returns the nonnegative file descriptor opened with the raw
/// [`sys::open`] semantics.
///
/// See [`sys::open`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`OpenError::Einval`]: the kernel rejected the supplied flag
///   combination.
/// - [`OpenError::Other`]: delegated pathname lookup, open, permission,
///   filesystem, security-hook, or descriptor-allocation error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn open(
    filename: &CStr,
    flags: Int,
    mode: UModeT,
) -> Result<Int, OpenError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pathname pointer for
    // the duration of the call.
    let ret = unsafe { sys::open(filename.as_ptr(), flags, mode) };

    result_from_ret(ret as isize, |ret| ret as Int, OpenError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, File},
        os::fd::FromRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Int, UModeT};

    use crate::Errno;

    use super::{OpenError, open};

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
    fn test_open_ok() {
        let path = temp_path("open");
        File::create(&path).unwrap();
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        let fd = open(path_cstr.as_c_str(), 0 as Int, 0 as UModeT)
            .expect("open should succeed for an existing temp file");
        let file = unsafe { File::from_raw_fd(fd) };
        drop(file);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_open_missing_path() {
        let path = temp_path("open_missing");
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            open(path_cstr.as_c_str(), 0 as Int, 0 as UModeT),
            Err(OpenError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_open_error_mapping() {
        assert_eq!(OpenError::from_errno(Errno::Einval), OpenError::Einval);
        assert_eq!(
            OpenError::from_errno(Errno::Enoent),
            OpenError::Other(Errno::Enoent)
        );
    }
}
