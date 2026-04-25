use celer_system_linux_ctypes::{Int, PidT};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`kill`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KillError {
    /// `EPERM`.
    Eperm,
    /// `ESRCH`.
    Esrch,
    /// `EINVAL`.
    Einval,
    /// Another errno returned by signal-audit or security-hook checks.
    Other(Errno),
}

impl KillError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Esrch => Self::Esrch,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Send a signal to a process or process group.
///
/// This safe wrapper preserves the raw `pid` and `sig` selector integers and
/// maps the raw `kill(2)` return value into `Result<(), KillError>`.
///
/// On success, returns `Ok(())` after the kernel has completed the requested
/// permission and target checks and, when `sig != 0`, queued the signal.
///
/// See [`sys::kill`] for kernel behavior, historical notes, reachable errors,
/// and source references.
///
/// # Errors
/// - [`KillError::Eperm`]: the caller was not permitted to signal at least one
///   selected target.
/// - [`KillError::Esrch`]: no matching process or process group could be
///   found.
/// - [`KillError::Einval`]: `sig` was not a valid signal number.
/// - [`KillError::Other`]: another errno returned by signal-audit or
///   security-hook checks.
pub fn kill(pid: PidT, sig: Int) -> Result<(), KillError> {
    let ret = sys::kill(pid, sig);

    unit_from_ret(ret as isize, KillError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PidT};

    use crate::{Errno, sys};

    use super::{KillError, kill};

    #[test]
    fn test_kill_signal_zero_self() {
        assert_eq!(kill(sys::getpid(), 0 as Int), Ok(()));
    }

    #[test]
    fn test_kill_invalid_signal() {
        assert_eq!(kill(sys::getpid(), Int::MAX), Err(KillError::Einval));
    }

    #[test]
    fn test_kill_nonexistent_process() {
        assert_eq!(kill(PidT::MAX, 0 as Int), Err(KillError::Esrch));
    }

    #[test]
    fn test_kill_error_mapping() {
        assert_eq!(KillError::from_errno(Errno::Eperm), KillError::Eperm);
        assert_eq!(KillError::from_errno(Errno::Esrch), KillError::Esrch);
        assert_eq!(KillError::from_errno(Errno::Einval), KillError::Einval);
        assert_eq!(
            KillError::from_errno(Errno::Eio),
            KillError::Other(Errno::Eio)
        );
    }
}
