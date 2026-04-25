use celer_system_linux_ctypes::{Char, Int};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`sethostname`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SethostnameError {
    Eperm,
    Einval,
    Efault,
    Other(Errno),
}

impl SethostnameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Set the hostname for the current UTS namespace from raw bytes.
///
/// This safe wrapper passes `name.len()` as the raw signed byte count. The
/// bytes are not required to be NUL-terminated.
///
/// See [`sys::sethostname`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SethostnameError::Eperm`]: the caller lacks permission.
/// - [`SethostnameError::Einval`]: `name` is too long for the kernel limit.
/// - [`SethostnameError::Efault`]: the kernel could not read `name`.
/// - [`SethostnameError::Other`]: another syscall error reported by the raw
///   ABI.
pub fn sethostname(name: &[u8]) -> Result<(), SethostnameError> {
    let len =
        Int::try_from(name.len()).map_err(|_| SethostnameError::Einval)?;
    // SAFETY: `name` is readable for `len` bytes.
    let ret = unsafe { sys::sethostname(name.as_ptr().cast::<Char>(), len) };
    unit_from_ret(ret as isize, SethostnameError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{SethostnameError, sethostname};

    #[test]
    fn test_sethostname_too_long_or_permission_denied() {
        let err = sethostname(&[b'a'; 65]).unwrap_err();
        assert!(matches!(
            err,
            SethostnameError::Eperm | SethostnameError::Einval
        ));
    }

    #[test]
    fn test_sethostname_error_mapping() {
        assert_eq!(
            SethostnameError::from_errno(Errno::Eperm),
            SethostnameError::Eperm
        );
        assert_eq!(
            SethostnameError::from_errno(Errno::Einval),
            SethostnameError::Einval
        );
        assert_eq!(
            SethostnameError::from_errno(Errno::Efault),
            SethostnameError::Efault
        );
        assert_eq!(
            SethostnameError::from_errno(Errno::Eio),
            SethostnameError::Other(Errno::Eio)
        );
    }
}
