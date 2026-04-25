use celer_system_linux_ctypes::Int;

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

pub use sys::{SIG_DFL, SIG_IGN, SigHandler, sig_handler};

/// Errors returned by [`signal`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SignalError {
    Efault,
    Einval,
    Other(Errno),
}

impl SignalError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Install a legacy x86 signal handler for `sig`.
///
/// `Ok(handler)` is the previous legacy signal handler or disposition word.
///
/// See [`sys::signal`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// If `handler` names a user handler, that handler must remain valid for
/// signal delivery and use the ABI expected by this historical interface.
///
/// # Errors
/// - [`SignalError::Efault`]: Linux 1.0 rejected the handler address.
/// - [`SignalError::Einval`]: `sig` is invalid or targets an immutable signal
///   disposition.
/// - [`SignalError::Other`]: any other syscall error reported by the raw ABI.
#[cfg(target_arch = "x86")]
pub unsafe fn signal(
    sig: Int,
    handler: SigHandler,
) -> Result<SigHandler, SignalError> {
    // SAFETY: handler ABI requirements are upheld by the caller.
    let ret = unsafe { sys::signal(sig, handler) };
    result_from_ret(
        ret as isize,
        |ret| sys::sig_handler_from_raw(ret as _),
        SignalError::from_errno,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{SIG_DFL, SignalError, signal};
    use crate::Errno;
    use crate::sys::test_support::process_global_state_guard;

    const SIGUSR1: i32 = 10;

    struct RestoreSignal(super::SigHandler);

    impl Drop for RestoreSignal {
        fn drop(&mut self) {
            let _ = unsafe { signal(SIGUSR1, self.0) };
        }
    }

    #[test]
    fn test_signal_success_returns_previous_handler() {
        let _guard = process_global_state_guard();

        let previous = unsafe { signal(SIGUSR1, SIG_DFL) }
            .expect("signal(SIGUSR1, SIG_DFL) failed");
        let _restore = RestoreSignal(previous);
    }

    #[test]
    fn test_signal_invalid_signal() {
        let result = unsafe { signal(0, SIG_DFL) };
        assert_eq!(result, Err(SignalError::Einval));
    }

    #[test]
    fn test_signal_error_mapping() {
        assert_eq!(SignalError::from_errno(Errno::Efault), SignalError::Efault);
        assert_eq!(SignalError::from_errno(Errno::Einval), SignalError::Einval);
        assert_eq!(
            SignalError::from_errno(Errno::Eio),
            SignalError::Other(Errno::Eio)
        );
    }
}
