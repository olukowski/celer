use core::ffi::CStr;

use celer_system_linux_ctypes::OffT;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::UnsignedInt;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`truncate`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TruncateError {
    Efault,
    Enametoolong,
    Enoent,
    Enomem,
    Enotdir,
    Eacces,
    Erofs,
    Einval,
    Other(Errno),
}

impl TruncateError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enoent => Self::Enoent,
            Errno::Enomem => Self::Enomem,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eacces => Self::Eacces,
            Errno::Erofs => Self::Erofs,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Set the size of the file named by `path` to `length` bytes.
///
/// This safe wrapper takes `path` as a NUL-terminated [`CStr`] and maps the raw
/// return into `Result<(), TruncateError>`.
///
/// On success, the kernel has updated the file size.
///
/// See [`sys::truncate`] for kernel behavior, ABI history, reachable errors,
/// and source references.
///
/// # Errors
/// - [`TruncateError::Efault`]: the kernel could not read `path`.
/// - [`TruncateError::Enametoolong`]: the pathname is too long.
/// - [`TruncateError::Enoent`]: the path is empty or missing.
/// - [`TruncateError::Enomem`]: pathname allocation failed.
/// - [`TruncateError::Enotdir`]: traversal needed a directory.
/// - [`TruncateError::Eacces`]: traversal or truncation lacked permission.
/// - [`TruncateError::Erofs`]: the file is on a read-only filesystem.
/// - [`TruncateError::Einval`]: the requested length is invalid.
/// - [`TruncateError::Other`]: delegated symlink, notification, or filesystem
///   error.
pub fn truncate(path: &CStr, length: OffT) -> Result<(), TruncateError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::truncate(path.as_ptr(), length) };
    unit_from_ret(ret as isize, TruncateError::from_errno)
}

/// Set the size of the file named by `path` through the Linux 1.0 unsigned
/// `truncate` ABI.
///
/// This safe wrapper mirrors [`sys::linux_1_0::truncate`] and maps the raw
/// return into `Result<(), TruncateError>`.
///
/// See [`sys::linux_1_0::truncate`] for kernel behavior, ABI history,
/// reachable errors, and source references.
#[cfg(target_arch = "x86")]
pub fn truncate_1_0(
    path: &CStr,
    length: UnsignedInt,
) -> Result<(), TruncateError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::linux_1_0::truncate(path.as_ptr(), length) };
    unit_from_ret(ret as isize, TruncateError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, OpenOptions},
        io::Write as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::OffT;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::UnsignedInt;

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::truncate_1_0;
    use super::{TruncateError, truncate};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_truncate_{now}"));
        path
    }

    #[test]
    fn test_truncate_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(b"abcdef").unwrap();
        drop(file);
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        truncate(path_cstr.as_c_str(), 3 as OffT).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), 3);

        fs::remove_file(path).unwrap();
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_truncate_1_0_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(b"abcdef").unwrap();
        drop(file);
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        truncate_1_0(path_cstr.as_c_str(), 2 as UnsignedInt).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), 2);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_truncate_missing_path() {
        let path = CString::new("/celer-truncate-definitely-missing").unwrap();

        assert_eq!(
            truncate(path.as_c_str(), 0).unwrap_err(),
            TruncateError::Enoent
        );
    }

    #[test]
    fn test_truncate_error_mapping() {
        assert_eq!(
            TruncateError::from_errno(Errno::Efault),
            TruncateError::Efault
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Raw(36)),
            TruncateError::Enametoolong
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Enoent),
            TruncateError::Enoent
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Enomem),
            TruncateError::Enomem
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Enotdir),
            TruncateError::Enotdir
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Eacces),
            TruncateError::Eacces
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Erofs),
            TruncateError::Erofs
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Einval),
            TruncateError::Einval
        );
        assert_eq!(
            TruncateError::from_errno(Errno::Eio),
            TruncateError::Other(Errno::Eio)
        );
    }
}
