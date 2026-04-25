use core::ffi::CStr;

use celer_system_linux_ctypes::Utimbuf;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`utime`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UtimeError {
    Efault,
    Eperm,
    Erofs,
    Other(Errno),
}

impl UtimeError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Eperm => Self::Eperm,
            Errno::Erofs => Self::Erofs,
            errno => Self::Other(errno),
        }
    }
}

/// Update a file's access and modification timestamps.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and represents a
/// nullable timestamp pointer as `Option<&Utimbuf>`.
///
/// `Ok(())` means the kernel accepted the timestamp update.
///
/// See [`sys::utime`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`UtimeError::Efault`]: the kernel could not read `pathname` or `times`.
/// - [`UtimeError::Eperm`]: the caller is not permitted to set explicit
///   timestamps.
/// - [`UtimeError::Erofs`]: the target is on a read-only filesystem.
/// - [`UtimeError::Other`]: delegated pathname lookup, filesystem, or
///   security-hook error.
pub fn utime(
    pathname: &CStr,
    times: Option<&Utimbuf>,
) -> Result<(), UtimeError> {
    let times = times.map_or(core::ptr::null(), |times| times as *const _);
    // SAFETY: `pathname` is a readable NUL-terminated string and `times` is
    // either null or readable for one `Utimbuf`.
    let ret = unsafe { sys::utime(pathname.as_ptr(), times) };
    unit_from_ret(ret as isize, UtimeError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, OpenOptions},
        os::unix::fs::MetadataExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{TimeT, Utimbuf};

    use crate::Errno;

    use super::{UtimeError, utime};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_utime_{now}"));
        path
    }

    #[test]
    fn test_utime_ok() {
        let path = temp_path();
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        let c_path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        let times = Utimbuf {
            actime: 3 as TimeT,
            modtime: 4 as TimeT,
        };

        assert_eq!(utime(c_path.as_c_str(), Some(&times)), Ok(()));

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.atime(), 3);
        assert_eq!(metadata.mtime(), 4);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_utime_missing_path() {
        let path = CString::new("/definitely/not/a/real/celer-utime").unwrap();

        assert_eq!(
            utime(path.as_c_str(), None),
            Err(UtimeError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_utime_error_mapping() {
        assert_eq!(UtimeError::from_errno(Errno::Efault), UtimeError::Efault);
        assert_eq!(UtimeError::from_errno(Errno::Eperm), UtimeError::Eperm);
        assert_eq!(UtimeError::from_errno(Errno::Erofs), UtimeError::Erofs);
        assert_eq!(
            UtimeError::from_errno(Errno::Enoent),
            UtimeError::Other(Errno::Enoent)
        );
    }
}
