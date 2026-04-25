use core::ffi::CStr;

use celer_system_linux_ctypes::{UModeT, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`mknod`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MknodError {
    /// `EINVAL`.
    Einval,
    /// `EPERM`.
    Eperm,
    /// Another errno returned by delegated pathname lookup, filesystem create
    /// operations, or security hooks.
    Other(Errno),
}

impl MknodError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Create the node named by `pathname`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, keeps the raw
/// `mode` and device arguments, and maps the raw syscall return value into
/// `Result<(), MknodError>`.
///
/// On success, returns `Ok(())` after the kernel has created the requested
/// filesystem node.
///
/// See [`sys::mknod`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`MknodError::Einval`]: `mode` selects an unsupported node type.
/// - [`MknodError::Eperm`]: `mode` requests a directory through `mknod`, or
///   the caller was not allowed to create the requested node.
/// - [`MknodError::Other`]: delegated pathname lookup, filesystem create, or
///   security-hook error.
pub fn mknod(
    pathname: &CStr,
    mode: UModeT,
    dev: UnsignedInt,
) -> Result<(), MknodError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pathname pointer for
    // the duration of the call.
    let ret = unsafe { sys::mknod(pathname.as_ptr(), mode, dev) };

    unit_from_ret(ret as isize, MknodError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{UModeT, UnsignedInt};

    use crate::Errno;

    use super::{MknodError, mknod};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_mknod_{now}"));
        path
    }

    #[test]
    fn test_mknod_regular_file_ok() {
        let path = temp_path();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            mknod(path_c.as_c_str(), (0o100000 | 0o600) as UModeT, 0),
            Ok(())
        );

        assert!(fs::metadata(&path).unwrap().is_file());
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_mknod_directory_mode_returns_eperm() {
        let path = temp_path();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            mknod(path_c.as_c_str(), 0o040755 as UModeT, 0 as UnsignedInt),
            Err(MknodError::Eperm)
        );
    }

    #[test]
    fn test_mknod_error_mapping() {
        assert_eq!(MknodError::from_errno(Errno::Einval), MknodError::Einval);
        assert_eq!(MknodError::from_errno(Errno::Eperm), MknodError::Eperm);
        assert_eq!(
            MknodError::from_errno(Errno::Enoent),
            MknodError::Other(Errno::Enoent)
        );
    }
}
