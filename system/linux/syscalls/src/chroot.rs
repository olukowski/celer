use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`chroot`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChrootError {
    /// `ENOENT`.
    Enoent,
    /// `ENOTDIR`.
    Enotdir,
    /// `EPERM`.
    Eperm,
    /// Another errno returned by delegated pathname lookup, execute/search
    /// permission, or security-hook work.
    Other(Errno),
}

impl ChrootError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enoent => Self::Enoent,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Change the calling process's root directory to a NUL-terminated pathname.
///
/// This safe wrapper takes a [`CStr`] pathname and maps the raw syscall return
/// value into `Result<(), ChrootError>`.
///
/// On success, returns `Ok(())` after the kernel updates the calling process's
/// root directory.
///
/// See [`sys::chroot`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ChrootError::Enoent`]: the path does not exist.
/// - [`ChrootError::Enotdir`]: the resolved object is not a directory.
/// - [`ChrootError::Eperm`]: the caller lacks permission to change root.
/// - [`ChrootError::Other`]: delegated pathname lookup, execute/search
///   permission, or security-hook error.
pub fn chroot(pathname: &CStr) -> Result<(), ChrootError> {
    // SAFETY: the `CStr` argument guarantees a valid, NUL-terminated pathname
    // pointer for the duration of the call.
    let ret = unsafe { sys::chroot(pathname.as_ptr()) };

    unit_from_ret(ret as isize, ChrootError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        ffi::CString,
        fs::{self, File},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{ChrootError, chroot};

    fn temp_path(stem: &str) -> std::path::PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_{stem}_{now}"));
        path
    }

    #[test]
    fn test_chroot_enoent() {
        let path = temp_path("chroot_missing");
        let path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(chroot(path.as_c_str()), Err(ChrootError::Enoent));
    }

    #[test]
    fn test_chroot_enotdir() {
        let path = temp_path("chroot_file");
        File::create(&path).unwrap();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(chroot(path_c.as_c_str()), Err(ChrootError::Enotdir));

        fs::remove_file(&path).unwrap();
    }
}
