use core::{mem::MaybeUninit, ptr};

use celer_system_linux_ctypes::{Timeval, Timezone};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`gettimeofday`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GettimeofdayError {
    /// `EFAULT`.
    Efault,
    /// Another errno reported by the raw ABI.
    Other(Errno),
}

impl GettimeofdayError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Get the current time of day and optional kernel timezone state.
///
/// This safe wrapper turns each nullable raw output pointer into an
/// `Option<&mut MaybeUninit<_>>`. Passing `None` for either output preserves
/// the raw syscall's null-pointer behavior.
///
/// `Ok(())` means the kernel initialized each provided output slot.
///
/// See [`sys::gettimeofday`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`GettimeofdayError::Efault`]: the kernel could not write one of the
///   provided output slots.
/// - [`GettimeofdayError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn gettimeofday(
    mut tv: Option<&mut MaybeUninit<Timeval>>,
    mut tz: Option<&mut MaybeUninit<Timezone>>,
) -> Result<(), GettimeofdayError> {
    let tv = tv.as_mut().map_or(ptr::null_mut(), |tv| tv.as_mut_ptr());
    let tz = tz.as_mut().map_or(ptr::null_mut(), |tz| tz.as_mut_ptr());

    // SAFETY: each non-null pointer comes from a `MaybeUninit` output slot,
    // and null is a meaningful input for both raw syscall arguments.
    let ret = unsafe { sys::gettimeofday(tv, tz) };

    result_from_ret(ret as isize, |_| (), GettimeofdayError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{Timeval, Timezone};

    use super::{GettimeofdayError, gettimeofday};
    use crate::{Errno, sys};

    #[test]
    fn test_gettimeofday_both_outputs() {
        let mut tv = MaybeUninit::<Timeval>::uninit();
        let mut tz = MaybeUninit::<Timezone>::uninit();

        gettimeofday(Some(&mut tv), Some(&mut tz))
            .expect("wrapped gettimeofday failed");

        let tv = unsafe { tv.assume_init() };
        let tz = unsafe { tz.assume_init() };

        assert!(tv.tv_sec > 0, "tv_sec should be a positive epoch value");
        assert!(
            (0..1_000_000).contains(&tv.tv_usec),
            "tv_usec should be in [0, 1_000_000), got {}",
            tv.tv_usec
        );
        assert_ne!(tz.tz_minuteswest, -1);
        assert_ne!(tz.tz_dsttime, -1);
    }

    #[test]
    fn test_gettimeofday_tv_none() {
        let mut tz = MaybeUninit::<Timezone>::uninit();

        gettimeofday(None, Some(&mut tz)).expect("wrapped gettimeofday failed");
    }

    #[test]
    fn test_gettimeofday_tz_none() {
        let mut tv = MaybeUninit::<Timeval>::uninit();

        gettimeofday(Some(&mut tv), None).expect("wrapped gettimeofday failed");
    }

    #[test]
    fn test_gettimeofday_both_none() {
        gettimeofday(None, None).expect("wrapped gettimeofday failed");
    }

    #[test]
    fn test_gettimeofday_matches_raw_with_null_outputs() {
        let wrapped = gettimeofday(None, None);
        let raw = unsafe {
            sys::gettimeofday(core::ptr::null_mut(), core::ptr::null_mut())
        };

        assert_eq!(wrapped, Ok(()));
        assert_eq!(raw, 0);
    }

    #[test]
    fn test_gettimeofday_error_mapping() {
        assert_eq!(
            GettimeofdayError::from_errno(Errno::Efault),
            GettimeofdayError::Efault
        );
        assert_eq!(
            GettimeofdayError::from_errno(Errno::Enomem),
            GettimeofdayError::Other(Errno::Enomem)
        );
    }
}
