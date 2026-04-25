use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, PidT, Rusage};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`wait4`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Wait4Error {
    Echild,
    Efault,
    Einval,
    Esrch,
    Eintr,
    Other(Errno),
}

impl Wait4Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Raw(10) => Self::Echild,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Esrch => Self::Esrch,
            Errno::Eintr => Self::Eintr,
            errno => Self::Other(errno),
        }
    }
}

/// Wait for a child selected by `pid`.
///
/// This safe wrapper represents nullable output pointers as
/// `Option<&mut MaybeUninit<_>>` and maps the raw return into
/// `Result<PidT, Wait4Error>`.
///
/// On success, returns the reported child PID, or `0` when `WNOHANG` is set
/// and no matching child is waitable.
///
/// See [`sys::wait4`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Wait4Error::Echild`]: no matching child exists or remains waitable.
/// - [`Wait4Error::Efault`]: the kernel could not write an output slot.
/// - [`Wait4Error::Einval`]: `options` contains unsupported bits.
/// - [`Wait4Error::Esrch`]: `pid` is rejected by the kernel.
/// - [`Wait4Error::Eintr`]: the blocking wait was interrupted by a signal.
/// - [`Wait4Error::Other`]: any other errno reported by the raw ABI.
pub fn wait4(
    pid: PidT,
    stat_addr: Option<&mut MaybeUninit<Int>>,
    options: Int,
    ru: Option<&mut MaybeUninit<Rusage>>,
) -> Result<PidT, Wait4Error> {
    let stat_addr =
        stat_addr.map_or(core::ptr::null_mut(), MaybeUninit::as_mut_ptr);
    let ru = ru.map_or(core::ptr::null_mut(), MaybeUninit::as_mut_ptr);
    // SAFETY: both output pointers are either null or writable for one value.
    let ret = unsafe { sys::wait4(pid, stat_addr, options, ru) };
    result_from_ret(ret as isize, |ret| ret as PidT, Wait4Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::Int;

    use crate::Errno;

    use super::{Wait4Error, wait4};

    const WNOHANG: Int = 1;

    #[test]
    fn test_wait4_no_children() {
        assert_eq!(wait4(-1, None, WNOHANG, None), Err(Wait4Error::Echild));
    }

    #[test]
    fn test_wait4_invalid_options() {
        let mut status = MaybeUninit::<Int>::uninit();

        assert_eq!(
            wait4(-1, Some(&mut status), -1, None),
            Err(Wait4Error::Einval)
        );
    }

    #[test]
    fn test_wait4_error_mapping() {
        assert_eq!(Wait4Error::from_errno(Errno::Raw(10)), Wait4Error::Echild);
        assert_eq!(Wait4Error::from_errno(Errno::Efault), Wait4Error::Efault);
        assert_eq!(Wait4Error::from_errno(Errno::Einval), Wait4Error::Einval);
        assert_eq!(Wait4Error::from_errno(Errno::Esrch), Wait4Error::Esrch);
        assert_eq!(Wait4Error::from_errno(Errno::Eintr), Wait4Error::Eintr);
        assert_eq!(
            Wait4Error::from_errno(Errno::Eio),
            Wait4Error::Other(Errno::Eio)
        );
    }
}
