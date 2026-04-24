use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`fchdir`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FchdirError {
    /// `EBADF`.
    Ebadf,
    /// `EACCES`.
    Eacces,
    /// `ENOTDIR`.
    Enotdir,
    /// Another errno returned by delegated permission or filesystem work.
    Other(Errno),
}

impl FchdirError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Eacces => Self::Eacces,
            Errno::Enotdir => Self::Enotdir,
            errno => Self::Other(errno),
        }
    }
}

/// Change the calling process's current working directory to the directory
/// referenced by `fd`.
///
/// This safe wrapper maps the raw `fchdir(2)` return value into
/// `Result<(), FchdirError>` while keeping the file-descriptor argument as the
/// crate's integer fd type.
///
/// On success, returns `Ok(())` after the kernel switches the current working
/// directory to the directory referenced by `fd`.
///
/// See [`sys::fchdir`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`FchdirError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FchdirError::Eacces`]: the referenced directory cannot be searched.
/// - [`FchdirError::Enotdir`]: `fd` does not refer to a directory.
/// - [`FchdirError::Other`]: delegated permission or filesystem error.
pub fn fchdir(fd: Int) -> Result<(), FchdirError> {
    let ret = sys::fchdir(fd as _);

    unit_from_ret(ret as isize, FchdirError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        fs::OpenOptions,
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{FchdirError, fchdir};
    use crate::{Errno, sys::test_support::process_global_state_guard};

    fn create_temp_dir(label: &str) -> std::path::PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_wrap_fchdir_{label}_{now}"));
        fs::create_dir(&path).unwrap();

        path
    }

    #[test]
    fn test_fchdir_ok() {
        let _guard = process_global_state_guard();
        let original_dir = env::current_dir().unwrap();
        let path = create_temp_dir("success");
        let dir = OpenOptions::new().read(true).open(&path).unwrap();

        assert_eq!(fchdir(dir.as_raw_fd()), Ok(()));
        assert_eq!(env::current_dir().unwrap(), path);

        env::set_current_dir(&original_dir).unwrap();
        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_fchdir_ebadf() {
        assert_eq!(fchdir(-1), Err(FchdirError::Ebadf));
    }

    #[test]
    fn test_fchdir_enotdir() {
        let path = env::temp_dir().join(format!(
            "celer_wrap_fchdir_file_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        assert_eq!(fchdir(file.as_raw_fd()), Err(FchdirError::Enotdir));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchdir_error_mapping() {
        assert_eq!(FchdirError::from_errno(Errno::Ebadf), FchdirError::Ebadf);
        assert_eq!(
            FchdirError::from_errno(Errno::Enotdir),
            FchdirError::Enotdir
        );
        assert_eq!(FchdirError::from_errno(Errno::Eacces), FchdirError::Eacces);
        assert_eq!(
            FchdirError::from_errno(Errno::Enoent),
            FchdirError::Other(Errno::Enoent)
        );
    }
}
