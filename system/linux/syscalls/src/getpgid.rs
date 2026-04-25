use celer_system_linux_ctypes::PidT;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`getpgid`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetpgidError {
    /// `ESRCH`.
    Esrch,
    /// Another errno returned by an LSM hook or the raw ABI.
    Other(Errno),
}

impl GetpgidError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Esrch => Self::Esrch,
            errno => Self::Other(errno),
        }
    }
}

/// Return the process group ID of a process.
///
/// This safe wrapper maps the raw `getpgid(2)` return value into
/// `Result<PidT, GetpgidError>` while keeping the PID selector as the
/// kernel-facing integer type. Passing `0` asks for the calling process's
/// process group ID.
///
/// On success, returns a nonnegative process group ID.
///
/// See [`sys::getpgid`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`GetpgidError::Esrch`]: no task matches `pid`, or the kernel cannot
///   report a process group for the task selected by `pid`.
/// - [`GetpgidError::Other`]: another errno reported by an LSM hook or the raw
///   ABI.
pub fn getpgid(pid: PidT) -> Result<PidT, GetpgidError> {
    let ret = sys::getpgid(pid);

    result_from_ret(ret as isize, |ret| ret as PidT, GetpgidError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PidT;

    use super::{GetpgidError, getpgid};
    use crate::{Errno, sys};

    #[test]
    fn test_getpgid_zero_matches_raw() {
        assert_eq!(getpgid(0), Ok(sys::getpgid(0)));
    }

    #[test]
    fn test_getpgid_self_matches_raw() {
        let pid = sys::getpid();

        assert_eq!(getpgid(pid), Ok(sys::getpgid(pid)));
    }

    #[test]
    fn test_getpgid_nonexistent_process_returns_esrch() {
        assert_eq!(getpgid(PidT::MAX), Err(GetpgidError::Esrch));
    }

    #[test]
    fn test_getpgid_error_mapping() {
        assert_eq!(GetpgidError::from_errno(Errno::Esrch), GetpgidError::Esrch);
        assert_eq!(
            GetpgidError::from_errno(Errno::Eperm),
            GetpgidError::Other(Errno::Eperm)
        );
    }
}
