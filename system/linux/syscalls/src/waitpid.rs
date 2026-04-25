use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, PidT};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`waitpid`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WaitpidError {
    Echild,
    Eintr,
    Efault,
    Einval,
    Esrch,
    Other(Errno),
}

impl WaitpidError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Raw(10) => Self::Echild,
            Errno::Eintr => Self::Eintr,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Esrch => Self::Esrch,
            errno => Self::Other(errno),
        }
    }
}

/// Wait for a child process to change state through the x86 `waitpid` ABI.
///
/// This safe wrapper represents the nullable wait-status pointer as
/// `Option<&mut MaybeUninit<Int>>` and maps the raw return into
/// `Result<PidT, WaitpidError>`.
///
/// On success, returns the child PID.
///
/// See [`sys::waitpid`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`WaitpidError::Echild`]: no matching child exists or remains waitable.
/// - [`WaitpidError::Eintr`]: the wait was interrupted by a signal.
/// - [`WaitpidError::Efault`]: the kernel could not write the status slot.
/// - [`WaitpidError::Einval`]: `options` contains unsupported bits.
/// - [`WaitpidError::Esrch`]: `pid` is rejected by the kernel.
/// - [`WaitpidError::Other`]: any other errno reported by the raw ABI.
#[cfg(target_arch = "x86")]
pub fn waitpid(
    pid: PidT,
    stat_addr: Option<&mut MaybeUninit<Int>>,
    options: Int,
) -> Result<PidT, WaitpidError> {
    let stat_addr =
        stat_addr.map_or(core::ptr::null_mut(), MaybeUninit::as_mut_ptr);
    // SAFETY: `stat_addr` is either null or writable for one `Int`.
    let ret = unsafe { sys::waitpid(pid, stat_addr, options) };
    result_from_ret(ret as isize, |ret| ret as PidT, WaitpidError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::WaitpidError;

    #[cfg(target_arch = "x86")]
    use super::waitpid;

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_waitpid_no_children() {
        assert_eq!(waitpid(-1, None, 1), Err(WaitpidError::Echild));
    }

    #[test]
    fn test_waitpid_error_mapping() {
        assert_eq!(
            WaitpidError::from_errno(Errno::Raw(10)),
            WaitpidError::Echild
        );
        assert_eq!(WaitpidError::from_errno(Errno::Eintr), WaitpidError::Eintr);
        assert_eq!(
            WaitpidError::from_errno(Errno::Efault),
            WaitpidError::Efault
        );
        assert_eq!(
            WaitpidError::from_errno(Errno::Einval),
            WaitpidError::Einval
        );
        assert_eq!(WaitpidError::from_errno(Errno::Esrch), WaitpidError::Esrch);
        assert_eq!(
            WaitpidError::from_errno(Errno::Eio),
            WaitpidError::Other(Errno::Eio)
        );
    }
}
