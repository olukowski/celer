use celer_system_linux_ctypes::OldUidT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setuid16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Setuid16Error {
    Einval,
    Enomem,
    Eperm,
    Other(Errno),
}

impl Setuid16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Set the calling process's user ID through the legacy x86 `setuid16` ABI.
///
/// This safe wrapper preserves the raw 16-bit uid value and maps the raw
/// syscall return value into `Result<(), Setuid16Error>`.
///
/// See [`sys::setuid16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Setuid16Error::Einval`]: `uid` is invalid in the current namespace.
/// - [`Setuid16Error::Enomem`]: the kernel could not allocate credentials.
/// - [`Setuid16Error::Eperm`]: the requested uid change is not permitted.
/// - [`Setuid16Error::Other`]: another errno from credential or security
///   hooks.
#[cfg(target_arch = "x86")]
pub fn setuid16(uid: OldUidT) -> Result<(), Setuid16Error> {
    let ret = sys::setuid16(uid);
    unit_from_ret(ret as isize, Setuid16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Setuid16Error, setuid16};
    use crate::{Errno, getuid16};

    #[test]
    fn test_setuid16_current_uid() {
        assert_eq!(setuid16(getuid16()), Ok(()));
    }

    #[test]
    fn test_setuid16_error_mapping() {
        assert_eq!(
            Setuid16Error::from_errno(Errno::Einval),
            Setuid16Error::Einval
        );
        assert_eq!(
            Setuid16Error::from_errno(Errno::Enomem),
            Setuid16Error::Enomem
        );
        assert_eq!(
            Setuid16Error::from_errno(Errno::Eperm),
            Setuid16Error::Eperm
        );
        assert_eq!(
            Setuid16Error::from_errno(Errno::Eio),
            Setuid16Error::Other(Errno::Eio)
        );
    }
}
