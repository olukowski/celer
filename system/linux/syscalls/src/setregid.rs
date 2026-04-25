use celer_system_linux_ctypes::OldGidT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setregid16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Setregid16Error {
    Einval,
    Enomem,
    Enosys,
    Eperm,
    Other(Errno),
}

impl Setregid16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Enosys => Self::Enosys,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Set the real and/or effective group ID through the legacy x86
/// `setregid16` ABI.
///
/// This safe wrapper preserves the raw 16-bit ID values. Pass
/// `OldGidT::MAX` for either argument to request no change.
///
/// See [`sys::setregid16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Setregid16Error::Einval`]: a requested gid is invalid in the current
///   namespace.
/// - [`Setregid16Error::Enomem`]: the kernel could not allocate credentials.
/// - [`Setregid16Error::Enosys`]: the legacy ABI is not configured.
/// - [`Setregid16Error::Eperm`]: the requested ID change is not permitted.
/// - [`Setregid16Error::Other`]: another errno from credential or security
///   hooks.
#[cfg(target_arch = "x86")]
pub fn setregid16(rgid: OldGidT, egid: OldGidT) -> Result<(), Setregid16Error> {
    let ret = sys::setregid16(rgid, egid);
    unit_from_ret(ret as isize, Setregid16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{Setregid16Error, setregid16};
    use crate::{getegid16, getgid16};

    #[test]
    fn test_setregid16_current_ids() {
        assert_eq!(setregid16(getgid16(), getegid16()), Ok(()));
    }

    #[test]
    fn test_setregid16_error_mapping() {
        assert_eq!(
            Setregid16Error::from_errno(Errno::Einval),
            Setregid16Error::Einval
        );
        assert_eq!(
            Setregid16Error::from_errno(Errno::Enomem),
            Setregid16Error::Enomem
        );
        assert_eq!(
            Setregid16Error::from_errno(Errno::Enosys),
            Setregid16Error::Enosys
        );
        assert_eq!(
            Setregid16Error::from_errno(Errno::Eperm),
            Setregid16Error::Eperm
        );
        assert_eq!(
            Setregid16Error::from_errno(Errno::Eio),
            Setregid16Error::Other(Errno::Eio)
        );
    }
}
