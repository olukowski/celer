use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, Itimerval};

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`getitimer`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetitimerError {
    Einval,
    Efault,
    Other(Errno),
}

impl GetitimerError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Read one process interval timer into an uninitialized `Itimerval` slot.
///
/// This wrapper accepts a mutable [`MaybeUninit<Itimerval>`] output slot and
/// initializes it on success. The `which` argument is passed through unchanged.
///
/// `Ok(())` means the kernel wrote the timer state into `value`.
///
/// See [`sys::getitimer`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - [`GetitimerError::Einval`]: `which` is not a supported timer selector.
/// - [`GetitimerError::Efault`]: the kernel could not write one `Itimerval`
///   to `value`.
/// - [`GetitimerError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn getitimer(
    which: Int,
    value: &mut MaybeUninit<Itimerval>,
) -> Result<(), GetitimerError> {
    // SAFETY: `value` provides writable storage for one `Itimerval`.
    let ret = unsafe { sys::getitimer(which, value.as_mut_ptr()) };
    result_from_ret(ret as isize, |_| (), GetitimerError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{
        ITIMER_PROF, ITIMER_REAL, ITIMER_VIRTUAL, Itimerval,
    };

    use crate::sys;

    use super::{GetitimerError, getitimer};

    #[test]
    fn test_getitimer_real_succeeds() {
        let mut timer = MaybeUninit::<Itimerval>::uninit();
        getitimer(ITIMER_REAL, &mut timer).expect("wrapped getitimer failed");
    }

    #[test]
    fn test_getitimer_virtual_succeeds() {
        let mut timer = MaybeUninit::<Itimerval>::uninit();
        getitimer(ITIMER_VIRTUAL, &mut timer)
            .expect("wrapped getitimer failed");
    }

    #[test]
    fn test_getitimer_prof_succeeds() {
        let mut timer = MaybeUninit::<Itimerval>::uninit();
        getitimer(ITIMER_PROF, &mut timer).expect("wrapped getitimer failed");
    }

    #[test]
    fn test_getitimer_rejects_invalid_which() {
        let mut timer = MaybeUninit::<Itimerval>::uninit();
        let err = getitimer(3, &mut timer).unwrap_err();
        assert_eq!(err, GetitimerError::Einval);
    }

    #[test]
    fn test_getitimer_error_mapping() {
        assert_eq!(
            GetitimerError::from_errno(crate::Errno::Einval),
            GetitimerError::Einval
        );
        assert_eq!(
            GetitimerError::from_errno(crate::Errno::Efault),
            GetitimerError::Efault
        );
        assert_eq!(
            GetitimerError::from_errno(crate::Errno::Enomem),
            GetitimerError::Other(crate::Errno::Enomem)
        );
    }

    #[test]
    fn test_getitimer_matches_raw_on_invalid_which() {
        let mut wrapped_timer = MaybeUninit::<Itimerval>::uninit();
        let wrapped = getitimer(3, &mut wrapped_timer);

        let mut raw_timer = MaybeUninit::<Itimerval>::uninit();
        let raw = unsafe { sys::getitimer(3, raw_timer.as_mut_ptr()) };

        assert_eq!(wrapped, Err(GetitimerError::Einval));
        assert_eq!(raw, -22);
    }
}
