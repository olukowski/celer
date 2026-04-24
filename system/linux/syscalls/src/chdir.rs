use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`chdir`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChdirError {
    /// Another errno returned by delegated pathname lookup or permission work.
    Other(Errno),
}

impl ChdirError {
    fn from_errno(errno: Errno) -> Self {
        Self::Other(errno)
    }
}

/// Change the calling process's current working directory to `path`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// syscall return value into `Result<(), ChdirError>`.
///
/// On success, returns `Ok(())` after changing the calling process's current
/// working directory.
///
/// See [`sys::chdir`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ChdirError::Other`]: delegated pathname lookup or permission error.
pub fn chdir(path: &CStr) -> Result<(), ChdirError> {
    // SAFETY: the `CStr` argument guarantees a valid, NUL-terminated pathname
    // pointer for the duration of the syscall.
    let ret = unsafe { sys::chdir(path.as_ptr()) };

    unit_from_ret(ret as isize, ChdirError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{ChdirError, chdir};
    use crate::{Errno, sys::test_support::process_global_state_guard};

    fn create_temp_dir() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_wrap_chdir_{now}"));
        fs::create_dir(&path).unwrap();

        path
    }

    #[test]
    fn test_chdir_ok() {
        let _guard = process_global_state_guard();
        let original_dir = env::current_dir().unwrap();
        let path = create_temp_dir();
        let path_cstr =
            std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
                .unwrap();

        assert_eq!(chdir(path_cstr.as_c_str()), Ok(()));
        assert_eq!(env::current_dir().unwrap(), path);

        env::set_current_dir(&original_dir).unwrap();
        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_chdir_missing_path() {
        let path = std::ffi::CString::new(
            "/definitely/not/a/real/celer-wrap-chdir-directory",
        )
        .unwrap();

        assert_eq!(
            chdir(path.as_c_str()),
            Err(ChdirError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_chdir_error_mapping() {
        assert_eq!(
            ChdirError::from_errno(Errno::Enoent),
            ChdirError::Other(Errno::Enoent)
        );
    }
}
