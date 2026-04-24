use celer_system_linux_ctypes::{Int, Timex};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`adjtimex`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdjtimexError {
    Efault,
    Einval,
    Eperm,
    Enodev,
    Other(Errno),
}

impl AdjtimexError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enodev => Self::Enodev,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Read or adjust kernel clock discipline parameters through `struct timex`.
///
/// On success, returns the kernel's time-state code and leaves the updated
/// `timex` contents in `txc`.
///
/// # Errors
/// - `EFAULT`: the kernel could not copy the `timex` contents in or out.
/// - `EINVAL`: the supplied mode bits or parameter ranges are invalid.
/// - `EPERM`: the caller lacks permission to perform the requested update.
/// - `ENODEV`: the core clock is not valid.
pub fn adjtimex(txc: &mut Timex) -> Result<Int, AdjtimexError> {
    // SAFETY: `&mut Timex` guarantees a valid, writable user buffer for the
    // duration of the syscall call site.
    let ret = unsafe { sys::adjtimex(txc as *mut Timex) };

    result_from_ret(ret as isize, |ret| ret as Int, AdjtimexError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{ADJ_ADJTIME, ADJ_TICK, Timex};

    use super::{AdjtimexError, adjtimex};

    fn zero_timex() -> Timex {
        Timex {
            modes: 0,
            offset: 0,
            freq: 0,
            maxerror: 0,
            esterror: 0,
            status: 0,
            constant: 0,
            precision: 0,
            tolerance: 0,
            time: celer_system_linux_ctypes::Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            tick: 0,
            ppsfreq: 0,
            jitter: 0,
            shift: 0,
            stabil: 0,
            jitcnt: 0,
            calcnt: 0,
            errcnt: 0,
            stbcnt: 0,
            tai: 0,
            __padding: [0; 11],
        }
    }

    #[test]
    fn test_adjtimex_query_succeeds() {
        let mut txc = zero_timex();
        let state = adjtimex(&mut txc).expect("adjtimex query should succeed");

        assert!(matches!(state, 0..=5), "unexpected adjtimex state: {state}");
        assert!(txc.maxerror >= 0, "maxerror should be non-negative");
        assert!(txc.esterror >= 0, "esterror should be non-negative");
    }

    #[test]
    fn test_adjtimex_rejects_invalid_mode_mix() {
        let mut txc = zero_timex();
        txc.modes = ADJ_ADJTIME;

        assert_eq!(
            adjtimex(&mut txc),
            Err(AdjtimexError::Einval),
            "ADJ_ADJTIME without ADJ_OFFSET_SINGLESHOT should be invalid"
        );
    }

    #[test]
    fn test_adjtimex_rejects_unprivileged_tick_change() {
        let mut txc = zero_timex();
        txc.modes = ADJ_TICK;
        txc.tick = 10_000;

        assert_eq!(
            adjtimex(&mut txc),
            Err(AdjtimexError::Eperm),
            "unprivileged tick changes should be rejected"
        );
    }
}
