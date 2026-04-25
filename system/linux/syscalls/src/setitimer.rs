use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, Itimerval};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setitimer`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetitimerError {
    Einval,
    Efault,
    Other(Errno),
}

impl SetitimerError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Set an interval timer and optionally return its previous state.
///
/// This wrapper accepts an optional initialized input timer and an optional
/// uninitialized output slot for the previous timer value.
///
/// See [`sys::setitimer`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SetitimerError::Einval`]: `which` or timer field values are invalid.
/// - [`SetitimerError::Efault`]: the kernel could not read `value` or write
///   `old_value`.
/// - [`SetitimerError::Other`]: another syscall error reported by the raw ABI.
pub fn setitimer(
    which: Int,
    value: Option<&Itimerval>,
    old_value: Option<&mut MaybeUninit<Itimerval>>,
) -> Result<(), SetitimerError> {
    let value = value.map_or(core::ptr::null(), |value| value);
    let old_value = old_value
        .map_or(core::ptr::null_mut(), |old_value| old_value.as_mut_ptr());

    // SAFETY: `value` is either null or readable for one `Itimerval`; `old_value`
    // is either null or writable for one `Itimerval`.
    let ret = unsafe { sys::setitimer(which, value, old_value) };
    unit_from_ret(ret as isize, SetitimerError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{
        ITIMER_PROF, ITIMER_REAL, ITIMER_VIRTUAL, Itimerval, Timeval,
    };

    use crate::{Errno, sys::test_support::process_global_state_guard};

    use super::{SetitimerError, setitimer};

    fn zero_itimerval() -> Itimerval {
        Itimerval {
            it_interval: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            it_value: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        }
    }

    #[test]
    fn test_setitimer_zero_value_succeeds() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        setitimer(ITIMER_REAL, Some(&zero), None).unwrap();
    }

    #[test]
    fn test_setitimer_none_value_clears_timer_and_writes_old_value() {
        let _guard = process_global_state_guard();
        let mut old = MaybeUninit::<Itimerval>::uninit();

        setitimer(ITIMER_REAL, None, Some(&mut old)).unwrap();
    }

    #[test]
    fn test_setitimer_invalid_selector() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        assert_eq!(
            setitimer(3, Some(&zero), None),
            Err(SetitimerError::Einval)
        );
    }

    #[test]
    fn test_setitimer_supported_selectors_accept_zero_value() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        for which in [ITIMER_REAL, ITIMER_VIRTUAL, ITIMER_PROF] {
            setitimer(which, Some(&zero), None).unwrap();
        }
    }

    #[test]
    fn test_setitimer_error_mapping() {
        assert_eq!(
            SetitimerError::from_errno(Errno::Einval),
            SetitimerError::Einval
        );
        assert_eq!(
            SetitimerError::from_errno(Errno::Efault),
            SetitimerError::Efault
        );
        assert_eq!(
            SetitimerError::from_errno(Errno::Enomem),
            SetitimerError::Other(Errno::Enomem)
        );
    }
}
