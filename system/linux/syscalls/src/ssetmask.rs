use celer_system_linux_ctypes::OldSigsetT;

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`ssetmask`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SsetmaskError {
    Enosys,
    Other(Errno),
}

impl SsetmaskError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Replace the caller's legacy blocked-signal mask word.
///
/// This safe wrapper keeps the scalar mask argument and maps the raw return
/// into `Result<OldSigsetT, SsetmaskError>`.
///
/// On success, returns the previous legacy blocked-signal mask word.
///
/// See [`sys::ssetmask`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SsetmaskError::Enosys`]: the legacy syscall is not configured.
/// - [`SsetmaskError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
pub fn ssetmask(newmask: OldSigsetT) -> Result<OldSigsetT, SsetmaskError> {
    let ret = sys::ssetmask(newmask);
    result_from_ret(
        ret as isize,
        |ret| ret as OldSigsetT,
        SsetmaskError::from_errno,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{SsetmaskError, ssetmask};
    use crate::Errno;
    use crate::sys::test_support::process_global_state_guard;

    #[test]
    fn test_ssetmask_smoke() {
        let _guard = process_global_state_guard();

        let result = ssetmask(0);
        assert!(result.is_ok() || result == Err(SsetmaskError::Enosys));
    }

    #[test]
    fn test_ssetmask_error_mapping() {
        assert_eq!(
            SsetmaskError::from_errno(Errno::Enosys),
            SsetmaskError::Enosys
        );
        assert_eq!(
            SsetmaskError::from_errno(Errno::Eio),
            SsetmaskError::Other(Errno::Eio)
        );
    }
}
