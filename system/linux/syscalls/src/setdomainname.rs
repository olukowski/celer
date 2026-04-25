use celer_system_linux_ctypes::{Char, Int};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setdomainname`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetdomainnameError {
    Eperm,
    Einval,
    Efault,
    Other(Errno),
}

impl SetdomainnameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Set the domain name for the current UTS namespace from raw bytes.
///
/// This safe wrapper passes `name.len()` as the raw signed byte count. The
/// bytes are not required to be NUL-terminated.
///
/// See [`sys::setdomainname`] for kernel behavior, historical notes,
/// reachable errors, and source references.
///
/// # Errors
/// - [`SetdomainnameError::Eperm`]: the caller lacks permission.
/// - [`SetdomainnameError::Einval`]: `name` is too long for the kernel limit.
/// - [`SetdomainnameError::Efault`]: the kernel could not read `name`.
/// - [`SetdomainnameError::Other`]: another syscall error reported by the raw
///   ABI.
pub fn setdomainname(name: &[u8]) -> Result<(), SetdomainnameError> {
    let len =
        Int::try_from(name.len()).map_err(|_| SetdomainnameError::Einval)?;
    // SAFETY: `name` is readable for `len` bytes.
    let ret = unsafe { sys::setdomainname(name.as_ptr().cast::<Char>(), len) };
    unit_from_ret(ret as isize, SetdomainnameError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{SetdomainnameError, setdomainname};

    #[test]
    fn test_setdomainname_too_long_or_permission_denied() {
        let err = setdomainname(&[b'a'; 65]).unwrap_err();
        assert!(matches!(
            err,
            SetdomainnameError::Eperm | SetdomainnameError::Einval
        ));
    }

    #[test]
    fn test_setdomainname_error_mapping() {
        assert_eq!(
            SetdomainnameError::from_errno(Errno::Eperm),
            SetdomainnameError::Eperm
        );
        assert_eq!(
            SetdomainnameError::from_errno(Errno::Einval),
            SetdomainnameError::Einval
        );
        assert_eq!(
            SetdomainnameError::from_errno(Errno::Efault),
            SetdomainnameError::Efault
        );
        assert_eq!(
            SetdomainnameError::from_errno(Errno::Eio),
            SetdomainnameError::Other(Errno::Eio)
        );
    }
}
