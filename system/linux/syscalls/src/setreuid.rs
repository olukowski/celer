use celer_system_linux_ctypes::OldUidT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setreuid16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Setreuid16Error {
    Eagain,
    Einval,
    Enomem,
    Eperm,
    Other(Errno),
}

impl Setreuid16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eagain => Self::Eagain,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Set the real and/or effective user ID through the legacy x86
/// `setreuid16` ABI.
///
/// This safe wrapper preserves the raw 16-bit ID values. Pass
/// `OldUidT::MAX` for either argument to request no change.
///
/// See [`sys::setreuid16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Setreuid16Error::Eagain`]: the kernel could not allocate per-user
///   accounting state.
/// - [`Setreuid16Error::Einval`]: a requested uid is invalid in the current
///   namespace.
/// - [`Setreuid16Error::Enomem`]: the kernel could not allocate credentials.
/// - [`Setreuid16Error::Eperm`]: the requested ID change is not permitted.
/// - [`Setreuid16Error::Other`]: another errno from credential or security
///   hooks.
#[cfg(target_arch = "x86")]
pub fn setreuid16(ruid: OldUidT, euid: OldUidT) -> Result<(), Setreuid16Error> {
    let ret = sys::setreuid16(ruid, euid);
    unit_from_ret(ret as isize, Setreuid16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{Setreuid16Error, setreuid16};
    use crate::{geteuid16, getuid16};

    #[test]
    fn test_setreuid16_current_ids() {
        assert_eq!(setreuid16(getuid16(), geteuid16()), Ok(()));
    }

    #[test]
    fn test_setreuid16_error_mapping() {
        assert_eq!(
            Setreuid16Error::from_errno(Errno::Eagain),
            Setreuid16Error::Eagain
        );
        assert_eq!(
            Setreuid16Error::from_errno(Errno::Einval),
            Setreuid16Error::Einval
        );
        assert_eq!(
            Setreuid16Error::from_errno(Errno::Enomem),
            Setreuid16Error::Enomem
        );
        assert_eq!(
            Setreuid16Error::from_errno(Errno::Eperm),
            Setreuid16Error::Eperm
        );
        assert_eq!(
            Setreuid16Error::from_errno(Errno::Eio),
            Setreuid16Error::Other(Errno::Eio)
        );
    }
}
