use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`uselib`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UselibError {
    Efault,
    Enametoolong,
    Enoent,
    Enomem,
    Enotdir,
    Eacces,
    Emfile,
    Enfile,
    Enoexec,
    Enosys,
    Other(Errno),
}

impl UselibError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enoent => Self::Enoent,
            Errno::Enomem => Self::Enomem,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eacces => Self::Eacces,
            Errno::Emfile => Self::Emfile,
            Errno::Raw(23) => Self::Enfile,
            Errno::Enoexec => Self::Enoexec,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Load a shared library through the historical `uselib` ABI.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname and maps the raw
/// return into `Result<(), UselibError>`.
///
/// See [`sys::uselib`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`UselibError::Efault`]: the kernel could not read `library`.
/// - [`UselibError::Enametoolong`]: the pathname is too long.
/// - [`UselibError::Enoent`]: the pathname is empty or missing.
/// - [`UselibError::Enomem`]: pathname allocation failed.
/// - [`UselibError::Enotdir`]: traversal needed a directory.
/// - [`UselibError::Eacces`]: access or loader permission was denied.
/// - [`UselibError::Emfile`]: the process file table was full.
/// - [`UselibError::Enfile`]: the system file table was full.
/// - [`UselibError::Enoexec`]: no loader accepted the file.
/// - [`UselibError::Enosys`]: the running kernel left this syscall
///   unimplemented.
/// - [`UselibError::Other`]: another loader or filesystem errno.
#[cfg(target_arch = "x86")]
pub fn uselib(library: &CStr) -> Result<(), UselibError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::uselib(library.as_ptr()) };
    unit_from_ret(ret as isize, UselibError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use crate::Errno;

    use super::{UselibError, uselib};

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_uselib_missing_path() {
        let path =
            CString::new("/definitely/not/a/real/celer-uselib-library.so")
                .unwrap();

        assert!(matches!(
            uselib(path.as_c_str()),
            Err(UselibError::Enoent
                | UselibError::Enoexec
                | UselibError::Enosys)
        ));
    }

    #[test]
    fn test_uselib_error_mapping() {
        assert_eq!(UselibError::from_errno(Errno::Efault), UselibError::Efault);
        assert_eq!(
            UselibError::from_errno(Errno::Raw(36)),
            UselibError::Enametoolong
        );
        assert_eq!(UselibError::from_errno(Errno::Enoent), UselibError::Enoent);
        assert_eq!(UselibError::from_errno(Errno::Enomem), UselibError::Enomem);
        assert_eq!(
            UselibError::from_errno(Errno::Enotdir),
            UselibError::Enotdir
        );
        assert_eq!(UselibError::from_errno(Errno::Eacces), UselibError::Eacces);
        assert_eq!(UselibError::from_errno(Errno::Emfile), UselibError::Emfile);
        assert_eq!(
            UselibError::from_errno(Errno::Raw(23)),
            UselibError::Enfile
        );
        assert_eq!(
            UselibError::from_errno(Errno::Enoexec),
            UselibError::Enoexec
        );
        assert_eq!(UselibError::from_errno(Errno::Enosys), UselibError::Enosys);
        assert_eq!(
            UselibError::from_errno(Errno::Eio),
            UselibError::Other(Errno::Eio)
        );
    }
}
