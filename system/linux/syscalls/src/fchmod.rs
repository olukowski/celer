use celer_system_linux_ctypes::{Int, UModeT};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`fchmod`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FchmodError {
    /// `EBADF`.
    Ebadf,
    /// `ENOENT`.
    Enoent,
    /// `EPERM`.
    Eperm,
    /// `EROFS`.
    Erofs,
    /// Another errno returned by delegated filesystem, inode, or security-hook
    /// work.
    Other(Errno),
}

impl FchmodError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Enoent => Self::Enoent,
            Errno::Eperm => Self::Eperm,
            Errno::Erofs => Self::Erofs,
            errno => Self::Other(errno),
        }
    }
}

/// Change the mode bits of the file referenced by `fd`.
///
/// This safe wrapper maps the raw `fchmod(2)` return value into
/// `Result<(), FchmodError>` while keeping the file-descriptor argument as the
/// kernel-facing integer type.
///
/// On success, returns `Ok(())` after the kernel applies the requested mode
/// bits to the file referenced by `fd`.
///
/// See [`sys::fchmod`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`FchmodError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FchmodError::Enoent`]: `fd` names a file-table entry with no inode.
/// - [`FchmodError::Eperm`]: the caller was not allowed to change the mode.
/// - [`FchmodError::Erofs`]: the referenced inode is on a read-only
///   filesystem.
/// - [`FchmodError::Other`]: delegated filesystem, inode, or security-hook
///   error.
pub fn fchmod(fd: Int, mode: UModeT) -> Result<(), FchmodError> {
    let ret = sys::fchmod(fd, mode);

    unit_from_ret(ret as isize, FchmodError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        os::unix::fs::PermissionsExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UModeT;

    use crate::Errno;

    use super::{FchmodError, fchmod};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_fchmod_{now}"));
        path
    }

    #[test]
    fn test_fchmod_ok() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let fd = file.into_raw_fd();

        assert_eq!(fchmod(fd, 0o600 as UModeT), Ok(()));

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, 0o600);

        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchmod_ebadf() {
        assert_eq!(fchmod(-1, 0o600 as UModeT), Err(FchmodError::Ebadf));
    }

    #[test]
    fn test_fchmod_error_mapping() {
        assert_eq!(FchmodError::from_errno(Errno::Ebadf), FchmodError::Ebadf);
        assert_eq!(FchmodError::from_errno(Errno::Enoent), FchmodError::Enoent);
        assert_eq!(FchmodError::from_errno(Errno::Eperm), FchmodError::Eperm);
        assert_eq!(FchmodError::from_errno(Errno::Erofs), FchmodError::Erofs);
        assert_eq!(
            FchmodError::from_errno(Errno::Eacces),
            FchmodError::Other(Errno::Eacces)
        );
    }
}
