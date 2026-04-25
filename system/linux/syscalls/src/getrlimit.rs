use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Rlimit, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`getrlimit`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetrlimitError {
    /// `EINVAL`.
    Einval,
    /// `EFAULT`.
    Efault,
    /// Another errno reported by the raw ABI.
    Other(Errno),
}

impl GetrlimitError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Copy the current process limit for one resource into an output slot.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Rlimit>` and passes `resource` through unchanged.
///
/// `Ok(())` means the kernel initialized `rlim` with the current soft and hard
/// limits for `resource`.
///
/// See [`sys::getrlimit`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`GetrlimitError::Einval`]: `resource` is outside the kernel's supported
///   resource range.
/// - [`GetrlimitError::Efault`]: the kernel could not write one `Rlimit` to
///   `rlim`.
/// - [`GetrlimitError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn getrlimit(
    resource: UnsignedInt,
    rlim: &mut MaybeUninit<Rlimit>,
) -> Result<(), GetrlimitError> {
    // SAFETY: `rlim` provides writable storage for one `Rlimit`.
    let ret = unsafe { sys::getrlimit(resource, rlim.as_mut_ptr()) };

    result_from_ret(ret as isize, |_| (), GetrlimitError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{Rlimit, UnsignedInt};

    use super::{GetrlimitError, getrlimit};
    use crate::{Errno, sys};

    const RLIMIT_CPU: UnsignedInt = 0;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    #[test]
    fn test_getrlimit_cpu_success() {
        let mut rlim = MaybeUninit::<Rlimit>::uninit();

        getrlimit(RLIMIT_CPU, &mut rlim).expect("wrapped getrlimit failed");
        let rlim = unsafe { rlim.assume_init() };

        assert!(
            rlim.rlim_cur <= rlim.rlim_max,
            "soft limit should not exceed hard limit: {rlim:?}"
        );
    }

    #[test]
    fn test_getrlimit_invalid_resource() {
        let mut rlim = MaybeUninit::<Rlimit>::uninit();

        assert_eq!(
            getrlimit(CURRENT_RLIM_NLIMITS, &mut rlim),
            Err(GetrlimitError::Einval)
        );
    }

    #[test]
    fn test_getrlimit_matches_raw_on_invalid_resource() {
        let mut wrapped_rlim = MaybeUninit::<Rlimit>::uninit();
        let wrapped = getrlimit(CURRENT_RLIM_NLIMITS, &mut wrapped_rlim);

        let mut raw_rlim = MaybeUninit::<Rlimit>::uninit();
        let raw = unsafe {
            sys::getrlimit(CURRENT_RLIM_NLIMITS, raw_rlim.as_mut_ptr())
        };

        assert_eq!(wrapped, Err(GetrlimitError::Einval));
        assert_eq!(raw, -22);
    }

    #[test]
    fn test_getrlimit_error_mapping() {
        assert_eq!(
            GetrlimitError::from_errno(Errno::Einval),
            GetrlimitError::Einval
        );
        assert_eq!(
            GetrlimitError::from_errno(Errno::Efault),
            GetrlimitError::Efault
        );
        assert_eq!(
            GetrlimitError::from_errno(Errno::Enomem),
            GetrlimitError::Other(Errno::Enomem)
        );
    }
}
