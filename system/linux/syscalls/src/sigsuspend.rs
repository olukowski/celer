use celer_system_linux_ctypes::OldSigsetT;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`sigsuspend`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SigsuspendError {
    Eintr,
    Other(Errno),
}

impl SigsuspendError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eintr => Self::Eintr,
            errno => Self::Other(errno),
        }
    }
}

/// Atomically replace the legacy blocked-signal mask and wait for a handler.
///
/// This safe wrapper keeps the scalar legacy mask argument and maps the raw
/// return into `Result<(), SigsuspendError>`.
///
/// A successful `Ok(())` is not expected from the source-verified Linux entry
/// path; after a handler runs, the syscall returns `EINTR`.
///
/// See [`sys::sigsuspend`] for kernel behavior, signal-mask restoration, and
/// source references.
///
/// # Errors
/// - [`SigsuspendError::Eintr`]: a signal handler ran.
/// - [`SigsuspendError::Other`]: any other syscall error reported by the raw
///   ABI.
#[cfg(target_arch = "x86")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn sigsuspend(mask: OldSigsetT) -> Result<(), SigsuspendError> {
    let ret = sys::sigsuspend(mask);
    unit_from_ret(ret as isize, SigsuspendError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::SigsuspendError;
    use crate::Errno;

    #[test]
    fn test_sigsuspend_error_mapping() {
        assert_eq!(
            SigsuspendError::from_errno(Errno::Eintr),
            SigsuspendError::Eintr
        );
        assert_eq!(
            SigsuspendError::from_errno(Errno::Eio),
            SigsuspendError::Other(Errno::Eio)
        );
    }
}
