use celer_system_linux_ctypes::{Long, UnsignedLong};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`ptrace`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PtraceError {
    /// `ESRCH`.
    Esrch,
    /// `EPERM`.
    Eperm,
    /// `EIO`.
    Eio,
    /// `EFAULT`.
    Efault,
    /// Another errno returned by request-specific ptrace handling.
    Other(Errno),
}

impl PtraceError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Esrch => Self::Esrch,
            Errno::Eperm => Self::Eperm,
            Errno::Eio => Self::Eio,
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Control or inspect another process with `ptrace`.
///
/// This wrapper keeps the raw four-argument ptrace ABI and maps the raw
/// syscall return value into `Result<Long, PtraceError>`.
///
/// On success, returns the nonnegative or positive request-specific raw
/// ptrace result.
///
/// See [`sys::ptrace`] for kernel behavior, reachable errors, request-specific
/// argument rules, and source references.
///
/// # Safety
/// The caller must ensure that `request`, `pid`, `addr`, and `data` satisfy
/// the memory-safety requirements of the chosen ptrace request. Any pointer
/// encoded in `addr` or `data` must remain valid for the duration of the
/// syscall and obey that request's ABI.
///
/// # Errors
/// - [`PtraceError::Esrch`]: the target task does not exist.
/// - [`PtraceError::Eperm`]: the caller lacks permission for the request.
/// - [`PtraceError::Eio`]: the request is invalid for the target or a
///   request-specific access failed.
/// - [`PtraceError::Efault`]: a user pointer argument was inaccessible.
/// - [`PtraceError::Other`]: another request-specific ptrace error.
pub unsafe fn ptrace(
    request: Long,
    pid: Long,
    addr: UnsignedLong,
    data: UnsignedLong,
) -> Result<Long, PtraceError> {
    // SAFETY: forwarded from this wrapper's request-specific contract.
    let ret = unsafe { sys::ptrace(request, pid, addr, data) };

    result_from_ret(ret as isize, |ret| ret as Long, PtraceError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Long;

    use crate::Errno;

    use super::{PtraceError, ptrace};

    const INVALID_REQUEST: Long = 9999;

    #[test]
    fn test_ptrace_invalid_request() {
        let ret = unsafe { ptrace(INVALID_REQUEST, 0, 0, 0) };

        assert!(ret.is_err());
    }

    #[test]
    fn test_ptrace_error_mapping() {
        assert_eq!(PtraceError::from_errno(Errno::Esrch), PtraceError::Esrch);
        assert_eq!(PtraceError::from_errno(Errno::Eperm), PtraceError::Eperm);
        assert_eq!(PtraceError::from_errno(Errno::Eio), PtraceError::Eio);
        assert_eq!(PtraceError::from_errno(Errno::Efault), PtraceError::Efault);
        assert_eq!(
            PtraceError::from_errno(Errno::Einval),
            PtraceError::Other(Errno::Einval)
        );
    }
}
