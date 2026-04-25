use celer_system_linux_ctypes::PidT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setpgid`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetpgidError {
    Einval,
    Esrch,
    Eperm,
    Eacces,
    Other(Errno),
}

impl SetpgidError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Esrch => Self::Esrch,
            Errno::Eperm => Self::Eperm,
            Errno::Eacces => Self::Eacces,
            other => Self::Other(other),
        }
    }
}

/// Set a process group ID.
///
/// This safe wrapper preserves the raw `pid` and `pgid` selector values and
/// maps the raw syscall return value into `Result<(), SetpgidError>`.
///
/// See [`sys::setpgid`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SetpgidError::Einval`]: `pgid` is negative or the target task is not a
///   thread-group leader.
/// - [`SetpgidError::Esrch`]: `pid` does not identify an eligible task.
/// - [`SetpgidError::Eperm`]: session or leadership checks failed.
/// - [`SetpgidError::Eacces`]: the target has already executed a new program
///   image in the relevant parent/session branch.
/// - [`SetpgidError::Other`]: another errno from security hooks.
pub fn setpgid(pid: PidT, pgid: PidT) -> Result<(), SetpgidError> {
    let ret = sys::setpgid(pid, pgid);
    unit_from_ret(ret as isize, SetpgidError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{SetpgidError, setpgid};

    #[test]
    fn test_setpgid_negative_pgid() {
        assert_eq!(setpgid(0, -1), Err(SetpgidError::Einval));
    }

    #[test]
    fn test_setpgid_error_mapping() {
        assert_eq!(
            SetpgidError::from_errno(Errno::Einval),
            SetpgidError::Einval
        );
        assert_eq!(SetpgidError::from_errno(Errno::Esrch), SetpgidError::Esrch);
        assert_eq!(SetpgidError::from_errno(Errno::Eperm), SetpgidError::Eperm);
        assert_eq!(
            SetpgidError::from_errno(Errno::Eacces),
            SetpgidError::Eacces
        );
        assert_eq!(
            SetpgidError::from_errno(Errno::Eio),
            SetpgidError::Other(Errno::Eio)
        );
    }
}
