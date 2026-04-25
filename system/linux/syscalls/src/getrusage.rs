use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, Rusage};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`getrusage`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetrusageError {
    /// `EINVAL`.
    Einval,
    /// `EFAULT`.
    Efault,
    /// Another errno reported by the raw ABI.
    Other(Errno),
}

impl GetrusageError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Return resource-usage accounting into an output slot.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Rusage>` and passes `who` through unchanged.
///
/// `Ok(())` means the kernel initialized `ru` with the selected usage
/// counters.
///
/// See [`sys::getrusage`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`GetrusageError::Einval`]: `who` is not a supported resource-usage
///   selector.
/// - [`GetrusageError::Efault`]: the kernel could not write one `Rusage` to
///   `ru`.
/// - [`GetrusageError::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn getrusage(
    who: Int,
    ru: &mut MaybeUninit<Rusage>,
) -> Result<(), GetrusageError> {
    // SAFETY: `ru` provides writable storage for one `Rusage`.
    let ret = unsafe { sys::getrusage(who, ru.as_mut_ptr()) };

    result_from_ret(ret as isize, |_| (), GetrusageError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{RUSAGE_CHILDREN, RUSAGE_SELF, Rusage};

    use super::{GetrusageError, getrusage};
    use crate::{Errno, sys};

    #[test]
    fn test_getrusage_self_succeeds() {
        let mut usage = MaybeUninit::<Rusage>::uninit();

        getrusage(RUSAGE_SELF, &mut usage).expect("wrapped getrusage failed");
    }

    #[test]
    fn test_getrusage_children_succeeds() {
        let mut usage = MaybeUninit::<Rusage>::uninit();

        getrusage(RUSAGE_CHILDREN, &mut usage)
            .expect("wrapped getrusage failed");
    }

    #[test]
    fn test_getrusage_rejects_invalid_who() {
        let mut usage = MaybeUninit::<Rusage>::uninit();

        assert_eq!(getrusage(-2, &mut usage), Err(GetrusageError::Einval));
    }

    #[test]
    fn test_getrusage_matches_raw_on_invalid_who() {
        let mut wrapped_usage = MaybeUninit::<Rusage>::uninit();
        let wrapped = getrusage(-2, &mut wrapped_usage);

        let mut raw_usage = MaybeUninit::<Rusage>::uninit();
        let raw = unsafe { sys::getrusage(-2, raw_usage.as_mut_ptr()) };

        assert_eq!(wrapped, Err(GetrusageError::Einval));
        assert_eq!(raw, -22);
    }

    #[test]
    fn test_getrusage_error_mapping() {
        assert_eq!(
            GetrusageError::from_errno(Errno::Einval),
            GetrusageError::Einval
        );
        assert_eq!(
            GetrusageError::from_errno(Errno::Efault),
            GetrusageError::Efault
        );
        assert_eq!(
            GetrusageError::from_errno(Errno::Enomem),
            GetrusageError::Other(Errno::Enomem)
        );
    }
}
