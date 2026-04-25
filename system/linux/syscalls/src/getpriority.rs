use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`getpriority`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetpriorityError {
    /// `EINVAL`.
    Einval,
    /// `ESRCH`.
    Esrch,
    /// Another errno reported by the raw ABI.
    Other(Errno),
}

impl GetpriorityError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Esrch => Self::Esrch,
            errno => Self::Other(errno),
        }
    }
}

/// Return the highest matching scheduler priority for a process, process
/// group, or user selection.
///
/// This safe wrapper maps the raw `getpriority(2)` return value into
/// `Result<Int, GetpriorityError>` while keeping the selector and subject as
/// the kernel-facing integer types.
///
/// On success, current kernels return the encoded compatibility priority value
/// described by the raw syscall documentation.
///
/// See [`sys::getpriority`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`GetpriorityError::Einval`]: `which` is not `PRIO_PROCESS`,
///   `PRIO_PGRP`, or `PRIO_USER`.
/// - [`GetpriorityError::Esrch`]: no task matches the selected `which` / `who`
///   pair.
/// - [`GetpriorityError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn getpriority(which: Int, who: Int) -> Result<Int, GetpriorityError> {
    let ret = sys::getpriority(which, who);

    result_from_ret(
        ret as isize,
        |ret| ret as Int,
        GetpriorityError::from_errno,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PRIO_PROCESS};

    use super::{GetpriorityError, getpriority};
    use crate::{Errno, sys};

    #[test]
    fn test_getpriority_current_process_matches_raw() {
        assert_eq!(
            getpriority(PRIO_PROCESS, 0),
            Ok(sys::getpriority(PRIO_PROCESS, 0))
        );
    }

    #[test]
    fn test_getpriority_rejects_invalid_selector() {
        assert_eq!(getpriority(3, 0), Err(GetpriorityError::Einval));
    }

    #[test]
    fn test_getpriority_nonexistent_process_returns_esrch() {
        assert_eq!(
            getpriority(PRIO_PROCESS, Int::MAX),
            Err(GetpriorityError::Esrch)
        );
    }

    #[test]
    fn test_getpriority_error_mapping() {
        assert_eq!(
            GetpriorityError::from_errno(Errno::Einval),
            GetpriorityError::Einval
        );
        assert_eq!(
            GetpriorityError::from_errno(Errno::Esrch),
            GetpriorityError::Esrch
        );
        assert_eq!(
            GetpriorityError::from_errno(Errno::Eperm),
            GetpriorityError::Other(Errno::Eperm)
        );
    }
}
