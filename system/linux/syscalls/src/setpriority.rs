use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`setpriority`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetpriorityError {
    Eacces,
    Einval,
    Eperm,
    Esrch,
    Other(Errno),
}

impl SetpriorityError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eacces => Self::Eacces,
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            Errno::Esrch => Self::Esrch,
            errno => Self::Other(errno),
        }
    }
}

/// Set the nice value for selected processes.
///
/// This safe wrapper preserves the raw `which`, `who`, and `niceval` selector
/// values and maps the raw syscall return value into
/// `Result<(), SetpriorityError>`.
///
/// See [`sys::setpriority`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SetpriorityError::Eacces`]: the caller tried to lower a nice value
///   without sufficient privilege.
/// - [`SetpriorityError::Einval`]: `which` is outside the supported selector
///   range.
/// - [`SetpriorityError::Eperm`]: the caller lacks permission for a matched
///   task.
/// - [`SetpriorityError::Esrch`]: no task matched the selected target.
/// - [`SetpriorityError::Other`]: another errno from security hooks.
pub fn setpriority(
    which: Int,
    who: Int,
    niceval: Int,
) -> Result<(), SetpriorityError> {
    let ret = sys::setpriority(which, who, niceval);
    unit_from_ret(ret as isize, SetpriorityError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::PRIO_PROCESS;

    use super::{SetpriorityError, setpriority};
    use crate::Errno;

    #[test]
    fn test_setpriority_current_process_nice_19() {
        assert_eq!(setpriority(PRIO_PROCESS, 0, 19), Ok(()));
    }

    #[test]
    fn test_setpriority_invalid_selector() {
        assert_eq!(setpriority(3, 0, 0), Err(SetpriorityError::Einval));
    }

    #[test]
    fn test_setpriority_missing_process() {
        assert_eq!(
            setpriority(PRIO_PROCESS, -1, 0),
            Err(SetpriorityError::Esrch)
        );
    }

    #[test]
    fn test_setpriority_error_mapping() {
        assert_eq!(
            SetpriorityError::from_errno(Errno::Eacces),
            SetpriorityError::Eacces
        );
        assert_eq!(
            SetpriorityError::from_errno(Errno::Einval),
            SetpriorityError::Einval
        );
        assert_eq!(
            SetpriorityError::from_errno(Errno::Eperm),
            SetpriorityError::Eperm
        );
        assert_eq!(
            SetpriorityError::from_errno(Errno::Esrch),
            SetpriorityError::Esrch
        );
        assert_eq!(
            SetpriorityError::from_errno(Errno::Eio),
            SetpriorityError::Other(Errno::Eio)
        );
    }
}
