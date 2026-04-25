use celer_system_linux_ctypes::{Timeval, Timezone};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`settimeofday`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SettimeofdayError {
    Efault,
    Einval,
    Eperm,
    Other(Errno),
}

impl SettimeofdayError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Set the system wall clock and/or legacy timezone state.
///
/// This safe wrapper converts nullable raw input pointers into `Option`
/// references. `None` passes a null pointer for that argument.
///
/// See [`sys::settimeofday`] for kernel behavior, reachable errors, side
/// effects, and source references.
///
/// # Errors
/// - [`SettimeofdayError::Efault`]: the kernel could not read a non-null
///   argument.
/// - [`SettimeofdayError::Einval`]: a supplied time or timezone value is
///   invalid, or the timekeeping layer rejected the update.
/// - [`SettimeofdayError::Eperm`]: the caller lacks permission to set system
///   time.
/// - [`SettimeofdayError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn settimeofday(
    tv: Option<&Timeval>,
    tz: Option<&Timezone>,
) -> Result<(), SettimeofdayError> {
    let tv = tv.map_or(core::ptr::null(), |tv| tv as *const Timeval);
    let tz = tz.map_or(core::ptr::null(), |tz| tz as *const Timezone);
    // SAFETY: non-null pointers came from shared references readable for one
    // value; null pointers are accepted by this ABI.
    let ret = unsafe { sys::settimeofday(tv, tz) };
    unit_from_ret(ret as isize, SettimeofdayError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Timeval;

    use super::{SettimeofdayError, settimeofday};
    use crate::Errno;

    #[test]
    fn test_settimeofday_invalid_tv_usec() {
        let tv = Timeval {
            tv_sec: 0,
            tv_usec: 1_000_001,
        };

        assert_eq!(
            settimeofday(Some(&tv), None),
            Err(SettimeofdayError::Einval)
        );
    }

    #[test]
    fn test_settimeofday_error_mapping() {
        assert_eq!(
            SettimeofdayError::from_errno(Errno::Efault),
            SettimeofdayError::Efault
        );
        assert_eq!(
            SettimeofdayError::from_errno(Errno::Einval),
            SettimeofdayError::Einval
        );
        assert_eq!(
            SettimeofdayError::from_errno(Errno::Eperm),
            SettimeofdayError::Eperm
        );
        assert_eq!(
            SettimeofdayError::from_errno(Errno::Eio),
            SettimeofdayError::Other(Errno::Eio)
        );
    }
}
