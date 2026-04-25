use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, OldSigsetT};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

pub use sys::{SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK};

/// Errors returned by [`sigprocmask`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SigprocmaskError {
    Efault,
    Einval,
    Other(Errno),
}

impl SigprocmaskError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Examine or change the caller's legacy blocked-signal mask word.
///
/// This wrapper converts nullable raw pointers into `Option` references. Use
/// `None` for `set` to query without changing the mask, and `None` for `oset`
/// to skip returning the previous mask.
///
/// On success, any requested mask update has been applied and `oset`, when
/// present, has been initialized with the previous mask.
///
/// See [`sys::sigprocmask`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SigprocmaskError::Efault`]: the kernel could not read `set` or write
///   `oset`.
/// - [`SigprocmaskError::Einval`]: `how` is invalid while `set` is present.
/// - [`SigprocmaskError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
pub fn sigprocmask(
    how: Int,
    set: Option<&OldSigsetT>,
    oset: Option<&mut MaybeUninit<OldSigsetT>>,
) -> Result<(), SigprocmaskError> {
    let set = set.map_or(core::ptr::null(), |set| set as *const OldSigsetT);
    let oset = oset.map_or(core::ptr::null_mut(), |oset| oset.as_mut_ptr());
    // SAFETY: non-null pointers came from references with the required access.
    let ret = unsafe { sys::sigprocmask(how, set, oset) };
    unit_from_ret(ret as isize, SigprocmaskError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::OldSigsetT;

    use super::{SIG_BLOCK, SIG_SETMASK, SigprocmaskError, sigprocmask};
    use crate::Errno;
    use crate::sys::test_support::process_global_state_guard;

    const SIGUSR1: OldSigsetT = 10;

    fn signal_bit(sig: OldSigsetT) -> OldSigsetT {
        (1 as OldSigsetT) << (sig - 1)
    }

    #[test]
    fn test_sigprocmask_query_ok() {
        let mut old = MaybeUninit::uninit();

        assert_eq!(sigprocmask(999, None, Some(&mut old)), Ok(()));
    }

    #[test]
    fn test_sigprocmask_updates_and_reports_previous_mask() {
        let _guard = process_global_state_guard();
        let clear = 0 as OldSigsetT;
        let mut original = MaybeUninit::uninit();
        sigprocmask(SIG_SETMASK, Some(&clear), Some(&mut original)).unwrap();
        let original = unsafe { original.assume_init() };

        let requested = signal_bit(SIGUSR1);
        let mut old = MaybeUninit::uninit();
        assert_eq!(
            sigprocmask(SIG_BLOCK, Some(&requested), Some(&mut old)),
            Ok(())
        );
        assert_eq!(unsafe { old.assume_init() }, 0);

        let _ = sigprocmask(SIG_SETMASK, Some(&original), None);
    }

    #[test]
    fn test_sigprocmask_rejects_invalid_how_when_set_is_present() {
        let requested = signal_bit(SIGUSR1);

        assert_eq!(
            sigprocmask(99, Some(&requested), None),
            Err(SigprocmaskError::Einval)
        );
    }

    #[test]
    fn test_sigprocmask_error_mapping() {
        assert_eq!(
            SigprocmaskError::from_errno(Errno::Efault),
            SigprocmaskError::Efault
        );
        assert_eq!(
            SigprocmaskError::from_errno(Errno::Einval),
            SigprocmaskError::Einval
        );
        assert_eq!(
            SigprocmaskError::from_errno(Errno::Eio),
            SigprocmaskError::Other(Errno::Eio)
        );
    }
}
