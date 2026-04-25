#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::UnsignedInt;
use celer_system_linux_ctypes::{Int, OffT};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`ftruncate`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FtruncateError {
    /// `EBADF`.
    Ebadf,
    /// `EINVAL`.
    Einval,
    /// `EPERM`.
    Eperm,
    /// Another errno returned by delegated security, notification, or
    /// filesystem truncate work.
    Other(Errno),
}

impl FtruncateError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Truncate an open file descriptor to `length` bytes.
///
/// This safe wrapper maps the raw `ftruncate(2)` return value into
/// `Result<(), FtruncateError>` while keeping the length argument in the
/// kernel-facing `off_t`-shaped type.
///
/// On success, returns `Ok(())` after the kernel updates the file size.
///
/// See [`sys::ftruncate`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`FtruncateError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FtruncateError::Einval`]: `length` is negative, the file is not a
///   regular file, the descriptor is not open for writing, or the request
///   exceeds the legacy non-large-file limit for this entrypoint.
/// - [`FtruncateError::Eperm`]: the inode is append-only.
/// - [`FtruncateError::Other`]: delegated security, notification, or
///   filesystem truncate error.
pub fn ftruncate(fd: Int, length: OffT) -> Result<(), FtruncateError> {
    let ret = sys::ftruncate(fd as _, length);

    unit_from_ret(ret as isize, FtruncateError::from_errno)
}

/// Errors returned by [`crate::linux_1_0::ftruncate`].
#[cfg(target_arch = "x86")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Ftruncate1_0Error {
    /// `EBADF`.
    Ebadf,
    /// `ENOENT`.
    Enoent,
    /// `EACCES`.
    Eacces,
    /// Another errno returned by delegated Linux 1.0 truncate work.
    Other(Errno),
}

#[cfg(target_arch = "x86")]
impl Ftruncate1_0Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Enoent => Self::Enoent,
            Errno::Eacces => Self::Eacces,
            errno => Self::Other(errno),
        }
    }
}

/// Truncate an open file descriptor through the Linux 1.0 unsigned
/// `ftruncate` ABI.
///
/// This safe wrapper mirrors [`sys::linux_1_0::ftruncate`] and maps the raw
/// return value into `Result<(), Ftruncate1_0Error>`.
///
/// On success, returns `Ok(())` after the Linux 1.0 kernel updates the inode
/// size.
///
/// See [`sys::linux_1_0::ftruncate`] for kernel behavior, reachable errors,
/// and source references.
///
/// # Errors
/// - [`Ftruncate1_0Error::Ebadf`]: `fd` is outside the open-file table or
///   does not name an open file descriptor.
/// - [`Ftruncate1_0Error::Enoent`]: the file table entry has no inode
///   attached.
/// - [`Ftruncate1_0Error::Eacces`]: the descriptor is not open for writing,
///   or the target inode is a directory.
/// - [`Ftruncate1_0Error::Other`]: delegated Linux 1.0 truncate error.
#[cfg(target_arch = "x86")]
pub fn ftruncate_1_0(
    fd: Int,
    length: UnsignedInt,
) -> Result<(), Ftruncate1_0Error> {
    let ret = sys::linux_1_0::ftruncate(fd as _, length);

    unit_from_ret(ret as isize, Ftruncate1_0Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::OffT;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::UnsignedInt;

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::{Ftruncate1_0Error, ftruncate_1_0};
    use super::{FtruncateError, ftruncate};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_ftruncate_{now}"));
        path
    }

    #[test]
    fn test_ftruncate_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(b"truncate-me").unwrap();

        assert_eq!(ftruncate(file.as_raw_fd(), 4 as OffT), Ok(()));
        assert_eq!(fs::metadata(&path).unwrap().len(), 4);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_ftruncate_ebadf() {
        assert_eq!(ftruncate(-1, 0 as OffT), Err(FtruncateError::Ebadf));
    }

    #[test]
    fn test_ftruncate_einval_for_negative_length() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        assert_eq!(
            ftruncate(file.as_raw_fd(), -1 as OffT),
            Err(FtruncateError::Einval)
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_ftruncate_error_mapping() {
        assert_eq!(
            FtruncateError::from_errno(Errno::Ebadf),
            FtruncateError::Ebadf
        );
        assert_eq!(
            FtruncateError::from_errno(Errno::Einval),
            FtruncateError::Einval
        );
        assert_eq!(
            FtruncateError::from_errno(Errno::Eperm),
            FtruncateError::Eperm
        );
        assert_eq!(
            FtruncateError::from_errno(Errno::Enoent),
            FtruncateError::Other(Errno::Enoent)
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_ftruncate_1_0_ok() {
        let path = temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(b"truncate-me").unwrap();

        assert_eq!(ftruncate_1_0(file.as_raw_fd(), 3 as UnsignedInt), Ok(()));
        assert_eq!(fs::metadata(&path).unwrap().len(), 3);

        fs::remove_file(&path).unwrap();
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_ftruncate_1_0_ebadf() {
        assert_eq!(
            ftruncate_1_0(-1, 0 as UnsignedInt),
            Err(Ftruncate1_0Error::Ebadf)
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_ftruncate_1_0_error_mapping() {
        assert_eq!(
            Ftruncate1_0Error::from_errno(Errno::Ebadf),
            Ftruncate1_0Error::Ebadf
        );
        assert_eq!(
            Ftruncate1_0Error::from_errno(Errno::Enoent),
            Ftruncate1_0Error::Enoent
        );
        assert_eq!(
            Ftruncate1_0Error::from_errno(Errno::Eacces),
            Ftruncate1_0Error::Eacces
        );
        assert_eq!(
            Ftruncate1_0Error::from_errno(Errno::Einval),
            Ftruncate1_0Error::Other(Errno::Einval)
        );
    }
}
