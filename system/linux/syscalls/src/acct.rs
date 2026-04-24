use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`acct`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AcctError {
    /// `EPERM`.
    Eperm,
    /// Another errno returned by delegated pathname, file-opening, or
    /// accounting-file validation work.
    Other(Errno),
}

impl AcctError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Enable or disable process accounting through the historical `acct` slot.
///
/// This safe wrapper takes `None` for a null raw pathname and `Some(&CStr)` for
/// a NUL-terminated accounting-file pathname.
///
/// Passing `None` disables accounting for the current PID namespace.
/// Passing `Some(path)` enables accounting on the named file.
///
/// See [`sys::acct`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`AcctError::Eperm`]: the permission check failed.
/// - [`AcctError::Other`]: delegated pathname, file-opening, or
///   accounting-file validation error.
pub fn acct(name: Option<&CStr>) -> Result<(), AcctError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated string for the
    // duration of the syscall, and `None` maps to a null pointer.
    let ret =
        unsafe { sys::acct(name.map_or(core::ptr::null(), CStr::as_ptr)) };

    unit_from_ret(ret as isize, AcctError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use crate::Errno;

    use super::{AcctError, acct};

    #[test]
    fn test_acct_none() {
        match acct(None) {
            Ok(()) | Err(AcctError::Eperm) => {}
            Err(err) => panic!("acct(None) returned unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_acct_invalid_path() {
        let path =
            CString::new("/definitely/not/a/real/celer-acct-file").unwrap();

        match acct(Some(path.as_c_str())) {
            Err(AcctError::Eperm) => {}
            Err(AcctError::Other(_)) => {}
            Ok(()) => panic!("acct(Some(...)) unexpectedly succeeded"),
        }
    }

    #[test]
    fn test_acct_error_mapping() {
        assert_eq!(AcctError::from_errno(Errno::Eperm), AcctError::Eperm);
        assert_eq!(
            AcctError::from_errno(Errno::Enoent),
            AcctError::Other(Errno::Enoent)
        );
    }
}
