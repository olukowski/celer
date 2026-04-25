use core::mem::MaybeUninit;

use celer_system_linux_ctypes::OldSigsetT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`sigpending`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SigpendingError {
    Efault,
    Other(Errno),
}

impl SigpendingError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Return the caller's pending blocked-signal mask word.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldSigsetT>` and maps the raw return into
/// `Result<(), SigpendingError>`.
///
/// On success, the kernel has initialized `set` with the legacy pending
/// blocked-signal mask word.
///
/// See [`sys::sigpending`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SigpendingError::Efault`]: the kernel could not write the output mask.
/// - [`SigpendingError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
pub fn sigpending(
    set: &mut MaybeUninit<OldSigsetT>,
) -> Result<(), SigpendingError> {
    // SAFETY: `MaybeUninit<OldSigsetT>` provides writable storage for one
    // legacy signal mask word.
    let ret = unsafe { sys::sigpending(set.as_mut_ptr()) };
    unit_from_ret(ret as isize, SigpendingError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use super::{SigpendingError, sigpending};
    use crate::Errno;

    #[test]
    fn test_sigpending_ok() {
        let mut pending = MaybeUninit::uninit();

        assert_eq!(sigpending(&mut pending), Ok(()));
    }

    #[test]
    fn test_sigpending_error_mapping() {
        assert_eq!(
            SigpendingError::from_errno(Errno::Efault),
            SigpendingError::Efault
        );
        assert_eq!(
            SigpendingError::from_errno(Errno::Einval),
            SigpendingError::Other(Errno::Einval)
        );
    }
}
