#![cfg(target_arch = "x86")]

use celer_system_linux_ctypes::UnsignedLong;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`ioperm`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IopermError {
    /// `EINVAL`.
    Einval,
    /// `EPERM`.
    Eperm,
    /// `ENOMEM`.
    Enomem,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by the raw syscall.
    Other(Errno),
}

impl IopermError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            Errno::Enomem => Self::Enomem,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Enable or disable access to a range of x86 I/O ports for the calling
/// thread.
///
/// This safe wrapper preserves the raw port-range arguments and converts the
/// raw `ioperm(2)` return value into `Result<(), IopermError>`.
///
/// On success, returns `Ok(())` after the kernel has applied the requested
/// permission change to the calling thread.
///
/// See [`sys::ioperm`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`IopermError::Einval`]: `from + num` overflowed or exceeded the
///   kernel's I/O-permission bitmap range.
/// - [`IopermError::Eperm`]: enabling access was not permitted.
/// - [`IopermError::Enomem`]: the kernel could not allocate or duplicate the
///   per-thread bitmap.
/// - [`IopermError::Enosys`]: the running kernel left this syscall slot
///   unimplemented.
/// - [`IopermError::Other`]: another raw-syscall errno.
pub fn ioperm(
    from: UnsignedLong,
    num: UnsignedLong,
    turn_on: bool,
) -> Result<(), IopermError> {
    let ret = sys::ioperm(from, num, turn_on);

    unit_from_ret(ret as isize, IopermError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UnsignedLong;

    use crate::Errno;

    use super::{IopermError, ioperm};

    #[test]
    fn test_ioperm_zero_length_range() {
        let err = ioperm(0 as UnsignedLong, 0 as UnsignedLong, true)
            .expect_err("ioperm(0, 0, true) should not succeed");

        assert!(matches!(err, IopermError::Einval | IopermError::Enosys));
    }

    #[test]
    fn test_ioperm_out_of_bounds_range() {
        let err = ioperm(65_535 as UnsignedLong, 2 as UnsignedLong, false)
            .expect_err("an out-of-bounds ioperm range should not succeed");

        assert!(matches!(err, IopermError::Einval | IopermError::Enosys));
    }

    #[test]
    fn test_ioperm_error_mapping() {
        assert_eq!(IopermError::from_errno(Errno::Einval), IopermError::Einval);
        assert_eq!(IopermError::from_errno(Errno::Eperm), IopermError::Eperm);
        assert_eq!(IopermError::from_errno(Errno::Enomem), IopermError::Enomem);
        assert_eq!(IopermError::from_errno(Errno::Enosys), IopermError::Enosys);
        assert_eq!(
            IopermError::from_errno(Errno::Eio),
            IopermError::Other(Errno::Eio)
        );
    }
}
