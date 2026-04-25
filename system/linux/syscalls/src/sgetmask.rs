use celer_system_linux_ctypes::OldSigsetT;

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`sgetmask`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SgetmaskError {
    Enosys,
    Other(Errno),
}

impl SgetmaskError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Return the caller's legacy blocked-signal mask word.
///
/// This safe wrapper maps the raw syscall return value into
/// `Result<OldSigsetT, SgetmaskError>`.
///
/// See [`sys::sgetmask`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`SgetmaskError::Enosys`]: the legacy syscall is not configured.
/// - [`SgetmaskError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
pub fn sgetmask() -> Result<OldSigsetT, SgetmaskError> {
    let ret = sys::sgetmask();
    result_from_ret(
        ret as isize,
        |ret| ret as OldSigsetT,
        SgetmaskError::from_errno,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{SgetmaskError, sgetmask};
    use crate::Errno;

    #[test]
    fn test_sgetmask_smoke() {
        let result = sgetmask();
        assert!(result.is_ok() || result == Err(SgetmaskError::Enosys));
    }

    #[test]
    fn test_sgetmask_error_mapping() {
        assert_eq!(
            SgetmaskError::from_errno(Errno::Enosys),
            SgetmaskError::Enosys
        );
        assert_eq!(
            SgetmaskError::from_errno(Errno::Eio),
            SgetmaskError::Other(Errno::Eio)
        );
    }
}
