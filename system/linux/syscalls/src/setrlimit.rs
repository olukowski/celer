#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Rlimit as Linux10Rlimit;
use celer_system_linux_ctypes::{Rlimit, UnsignedInt};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setrlimit`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetrlimitError {
    Efault,
    Einval,
    Eperm,
    Other(Errno),
}

impl SetrlimitError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Set the current task's soft and hard resource limits for one resource.
///
/// This safe wrapper replaces the raw input pointer with `&Rlimit`.
///
/// See [`sys::setrlimit`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SetrlimitError::Efault`]: the kernel could not read `rlim`.
/// - [`SetrlimitError::Einval`]: `resource` is unsupported or the requested
///   limits are malformed.
/// - [`SetrlimitError::Eperm`]: the requested update exceeds the caller's
///   authority.
/// - [`SetrlimitError::Other`]: another errno from security hooks.
pub fn setrlimit(
    resource: UnsignedInt,
    rlim: &Rlimit,
) -> Result<(), SetrlimitError> {
    // SAFETY: `rlim` is readable for one `Rlimit`.
    let ret = unsafe { sys::setrlimit(resource, rlim) };
    unit_from_ret(ret as isize, SetrlimitError::from_errno)
}

/// Set Linux 1.0 soft and hard resource limits through the historical x86 ABI.
///
/// This safe wrapper replaces the raw input pointer with
/// `&linux_1_0::Rlimit`.
///
/// See [`sys::linux_1_0::setrlimit`] for kernel behavior, reachable errors,
/// and source references.
///
/// # Errors
/// - [`SetrlimitError::Einval`]: `resource` is outside Linux 1.0's resource
///   table.
/// - [`SetrlimitError::Eperm`]: the requested update exceeds the caller's
///   authority.
#[cfg(target_arch = "x86")]
pub fn setrlimit_1_0(
    resource: UnsignedInt,
    rlim: &Linux10Rlimit,
) -> Result<(), SetrlimitError> {
    // SAFETY: `rlim` is readable for one historical `Rlimit`.
    let ret = unsafe { sys::linux_1_0::setrlimit(resource, rlim) };
    unit_from_ret(ret as isize, SetrlimitError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::linux_1_0::Rlimit as Linux10Rlimit;
    use celer_system_linux_ctypes::{Rlimit, UnsignedInt};

    #[cfg(target_arch = "x86")]
    use super::setrlimit_1_0;
    use super::{SetrlimitError, setrlimit};
    use crate::Errno;

    const RLIMIT_CPU: UnsignedInt = 0;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    #[test]
    fn test_setrlimit_invalid_resource() {
        let limits = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        assert_eq!(
            setrlimit(CURRENT_RLIM_NLIMITS, &limits),
            Err(SetrlimitError::Einval)
        );
    }

    #[test]
    fn test_setrlimit_rejects_soft_limit_above_hard_limit() {
        let limits = Rlimit {
            rlim_cur: 1,
            rlim_max: 0,
        };

        assert_eq!(setrlimit(RLIMIT_CPU, &limits), Err(SetrlimitError::Einval));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_setrlimit_1_0_invalid_resource() {
        let limits = Linux10Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        assert_eq!(
            setrlimit_1_0(CURRENT_RLIM_NLIMITS, &limits),
            Err(SetrlimitError::Einval)
        );
    }

    #[test]
    fn test_setrlimit_error_mapping() {
        assert_eq!(
            SetrlimitError::from_errno(Errno::Efault),
            SetrlimitError::Efault
        );
        assert_eq!(
            SetrlimitError::from_errno(Errno::Einval),
            SetrlimitError::Einval
        );
        assert_eq!(
            SetrlimitError::from_errno(Errno::Eperm),
            SetrlimitError::Eperm
        );
        assert_eq!(
            SetrlimitError::from_errno(Errno::Eio),
            SetrlimitError::Other(Errno::Eio)
        );
    }
}
