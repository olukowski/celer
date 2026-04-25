use celer_system_linux_ctypes::TimeT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`stime`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StimeError {
    Eperm,
    Efault,
    Einval,
    Other(Errno),
}

impl StimeError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Set the system clock to a whole-second Unix timestamp.
///
/// This safe wrapper replaces the raw pointer with `&TimeT` and maps the raw
/// return into `Result<(), StimeError>`.
///
/// On success, the kernel time-setting path accepted the request.
///
/// See [`sys::stime`] for kernel behavior, required privileges, reachable
/// errors, and source references.
///
/// # Errors
/// - [`StimeError::Eperm`]: the caller lacks permission to set the system
///   time.
/// - [`StimeError::Efault`]: the kernel could not read the timestamp.
/// - [`StimeError::Einval`]: the timestamp was rejected by the time-setting
///   path.
/// - [`StimeError::Other`]: any other syscall error reported by the raw ABI.
#[cfg(target_arch = "x86")]
pub fn stime(tptr: &TimeT) -> Result<(), StimeError> {
    // SAFETY: `tptr` is readable for one `TimeT`.
    let ret = unsafe { sys::stime(tptr as *const TimeT) };
    unit_from_ret(ret as isize, StimeError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::TimeT;

    use super::{StimeError, stime};
    use crate::Errno;

    #[test]
    fn test_stime_requires_permission_or_valid_time() {
        let invalid = -1 as TimeT;
        let result = stime(&invalid);

        assert!(matches!(
            result,
            Err(StimeError::Eperm | StimeError::Einval)
        ));
    }

    #[test]
    fn test_stime_error_mapping() {
        assert_eq!(StimeError::from_errno(Errno::Eperm), StimeError::Eperm);
        assert_eq!(StimeError::from_errno(Errno::Efault), StimeError::Efault);
        assert_eq!(StimeError::from_errno(Errno::Einval), StimeError::Einval);
        assert_eq!(
            StimeError::from_errno(Errno::Eio),
            StimeError::Other(Errno::Eio)
        );
    }
}
