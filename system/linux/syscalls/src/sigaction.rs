use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, OldSigaction};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`sigaction`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SigactionError {
    Efault,
    Einval,
    Other(Errno),
}

impl SigactionError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Get or set the legacy x86 signal action for `sig`.
///
/// This wrapper converts nullable raw pointers into `Option` references. Use
/// `None` for `action` to query without installing a new action, and `None`
/// for `oldaction` to skip returning the previous action.
///
/// See [`sys::sigaction`] for kernel behavior, reachable errors, side
/// effects, and source references.
///
/// # Safety
/// If `action` installs a user handler or restorer address, that callback
/// state must remain valid for future signal delivery and use the ABI expected
/// by this historical interface.
///
/// # Errors
/// - [`SigactionError::Efault`]: the kernel could not read `action` or write
///   `oldaction`.
/// - [`SigactionError::Einval`]: `sig` is invalid or `action` tries to change
///   an immutable signal disposition.
/// - [`SigactionError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
pub unsafe fn sigaction(
    sig: Int,
    action: Option<&OldSigaction>,
    oldaction: Option<&mut MaybeUninit<OldSigaction>>,
) -> Result<(), SigactionError> {
    let action = action
        .map_or(core::ptr::null(), |action| action as *const OldSigaction);
    let oldaction = oldaction
        .map_or(core::ptr::null_mut(), |oldaction| oldaction.as_mut_ptr());
    // SAFETY: non-null pointers came from references with the required access;
    // callback ABI requirements are upheld by the caller.
    let ret = unsafe { sys::sigaction(sig, action, oldaction) };
    unit_from_ret(ret as isize, SigactionError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::OldSigaction;

    use super::{SigactionError, sigaction};
    use crate::Errno;

    const SIGKILL: i32 = 9;
    const SIGUSR1: i32 = 10;
    const SIG_IGN: usize = 1;

    #[test]
    fn test_sigaction_query() {
        let mut old = MaybeUninit::<OldSigaction>::uninit();
        let result = unsafe { sigaction(SIGUSR1, None, Some(&mut old)) };
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_sigaction_rejects_sigkill_handler_install() {
        let ignore = OldSigaction {
            sa_handler: SIG_IGN,
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };

        let result = unsafe { sigaction(SIGKILL, Some(&ignore), None) };
        assert_eq!(result, Err(SigactionError::Einval));
    }

    #[test]
    fn test_sigaction_error_mapping() {
        assert_eq!(
            SigactionError::from_errno(Errno::Efault),
            SigactionError::Efault
        );
        assert_eq!(
            SigactionError::from_errno(Errno::Einval),
            SigactionError::Einval
        );
        assert_eq!(
            SigactionError::from_errno(Errno::Eio),
            SigactionError::Other(Errno::Eio)
        );
    }
}
