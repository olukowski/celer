use celer_system_linux_ctypes::PidT;

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setsid`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetsidError {
    Eperm,
    Other(Errno),
}

impl SetsidError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Create a new session and make the caller its leader.
///
/// `Ok(pid)` is the new process group ID returned by the kernel.
///
/// See [`sys::setsid`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SetsidError::Eperm`]: the caller is already a session leader or a
///   process group already exists with the proposed session ID.
/// - [`SetsidError::Other`]: any other syscall error reported by the raw ABI.
pub fn setsid() -> Result<PidT, SetsidError> {
    let ret = sys::setsid();
    result_from_ret(ret as isize, |ret| ret as PidT, SetsidError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{SetsidError, setsid};
    use crate::Errno;

    #[test]
    fn test_setsid_smoke() {
        let ret = setsid();
        assert!(
            matches!(ret, Ok(pid) if pid > 0) || ret == Err(SetsidError::Eperm)
        );
    }

    #[test]
    fn test_setsid_error_mapping() {
        assert_eq!(SetsidError::from_errno(Errno::Eperm), SetsidError::Eperm);
        assert_eq!(
            SetsidError::from_errno(Errno::Einval),
            SetsidError::Other(Errno::Einval)
        );
    }
}
