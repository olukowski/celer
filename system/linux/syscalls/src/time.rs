use core::mem::MaybeUninit;

use celer_system_linux_ctypes::TimeT;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`time`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeError {
    Efault,
    Other(Errno),
}

impl TimeError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Return the current calendar time in seconds since the Unix epoch.
///
/// This safe wrapper replaces the nullable raw output pointer with
/// `Option<&mut MaybeUninit<TimeT>>` and maps the raw return into
/// `Result<TimeT, TimeError>`.
///
/// On success, returns the current time. When `tloc` is `Some`, the kernel has
/// also initialized that slot with the same value.
///
/// See [`sys::time`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`TimeError::Efault`]: the kernel could not write the optional output.
/// - [`TimeError::Other`]: any other syscall error reported by the raw ABI.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn time(tloc: Option<&mut MaybeUninit<TimeT>>) -> Result<TimeT, TimeError> {
    let ptr = tloc
        .map(|tloc| tloc.as_mut_ptr())
        .unwrap_or(core::ptr::null_mut());

    // SAFETY: non-null pointers are writable for one `TimeT`; null is an
    // explicitly supported input.
    let ret = unsafe { sys::time(ptr) };
    result_from_ret(ret as isize, |ret| ret as TimeT, TimeError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use crate::Errno;

    use super::{TimeError, time};

    #[test]
    fn test_time_null_ok() {
        assert!(time(None).unwrap() > 0);
    }

    #[test]
    fn test_time_writes_output() {
        let mut stored = MaybeUninit::uninit();
        let now = time(Some(&mut stored)).unwrap();

        // SAFETY: `time` returned success, so the kernel initialized `stored`.
        let stored = unsafe { stored.assume_init() };
        assert_eq!(now, stored);
    }

    #[test]
    fn test_time_error_mapping() {
        assert_eq!(TimeError::from_errno(Errno::Efault), TimeError::Efault);
        assert_eq!(
            TimeError::from_errno(Errno::Eio),
            TimeError::Other(Errno::Eio)
        );
    }
}
