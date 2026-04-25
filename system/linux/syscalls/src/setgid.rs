use celer_system_linux_ctypes::OldGidT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setgid16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Setgid16Error {
    Einval,
    Enomem,
    Eperm,
    Other(Errno),
}

impl Setgid16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Eperm => Self::Eperm,
            other => Self::Other(other),
        }
    }
}

/// Set the calling process's group ID through the legacy x86 `setgid16` ABI.
///
/// This safe wrapper preserves the raw 16-bit gid argument and maps the raw
/// syscall return value into `Result<(), Setgid16Error>`.
///
/// See [`sys::setgid16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Setgid16Error::Einval`]: the gid is invalid in the current user
///   namespace.
/// - [`Setgid16Error::Enomem`]: the kernel could not allocate credentials.
/// - [`Setgid16Error::Eperm`]: the caller lacks permission for the requested
///   transition.
/// - [`Setgid16Error::Other`]: another errno from credential or security
///   checks.
#[cfg(target_arch = "x86")]
pub fn setgid16(gid: OldGidT) -> Result<(), Setgid16Error> {
    let ret = sys::setgid16(gid);
    unit_from_ret(ret as isize, Setgid16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{Errno, sys};

    use super::{Setgid16Error, setgid16};

    #[test]
    fn test_setgid16_current_gid() {
        assert_eq!(setgid16(sys::getgid16()), Ok(()));
    }

    #[test]
    fn test_setgid16_error_mapping() {
        assert_eq!(
            Setgid16Error::from_errno(Errno::Einval),
            Setgid16Error::Einval
        );
        assert_eq!(
            Setgid16Error::from_errno(Errno::Enomem),
            Setgid16Error::Enomem
        );
        assert_eq!(
            Setgid16Error::from_errno(Errno::Eperm),
            Setgid16Error::Eperm
        );
        assert_eq!(
            Setgid16Error::from_errno(Errno::Eio),
            Setgid16Error::Other(Errno::Eio)
        );
    }
}
