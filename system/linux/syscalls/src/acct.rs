use core::ffi::CStr;

use crate::errno::Errno;
use crate::sys;

/// Errors returned by [`acct`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AcctError {
    Eperm,
    Other(Errno),
}

impl From<isize> for AcctError {
    fn from(value: isize) -> Self {
        match Errno::from(value) {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Enable or disable process accounting through the historical `acct` slot.
///
/// Passing `None` disables accounting for the current PID namespace.
/// Passing `Some(path)` enables accounting on the named file.
///
/// # Errors
/// - `EPERM`: the caller lacks `CAP_SYS_PACCT`.
/// - `Other(..)`: pathname resolution, file opening, or accounting-file
///   validation returned a delegated errno.
pub fn acct(name: Option<&CStr>) -> Result<(), AcctError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated string for the
    // duration of the syscall, and `None` maps to a null pointer.
    let ret =
        unsafe { sys::acct(name.map_or(core::ptr::null(), CStr::as_ptr)) };

    if Errno::is_errno(ret) {
        Err(AcctError::from(ret))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

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
}
