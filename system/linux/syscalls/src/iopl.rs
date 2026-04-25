#![cfg(target_arch = "x86")]

use celer_system_linux_ctypes::UnsignedInt;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`iopl`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IoplError {
    /// `EINVAL`.
    Einval,
    /// `EPERM`.
    Eperm,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by the raw syscall.
    Other(Errno),
}

impl IoplError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Eperm => Self::Eperm,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Set the calling thread's x86 I/O privilege level.
///
/// This safe wrapper preserves the raw integer level argument and converts the
/// raw `iopl(2)` return value into `Result<(), IoplError>`.
///
/// On success, returns `Ok(())` after the kernel has applied the requested
/// I/O privilege level to the calling thread.
///
/// See [`sys::iopl`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`IoplError::Einval`]: `level` was outside the kernel's accepted range.
/// - [`IoplError::Eperm`]: raising the effective I/O privilege level was not
///   permitted.
/// - [`IoplError::Enosys`]: the running kernel left this syscall slot
///   unimplemented.
/// - [`IoplError::Other`]: another raw-syscall errno.
pub fn iopl(level: UnsignedInt) -> Result<(), IoplError> {
    let ret = sys::iopl(level);

    unit_from_ret(ret as isize, IoplError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UnsignedInt;

    use crate::Errno;

    use super::{IoplError, iopl};

    #[test]
    fn test_iopl_invalid_level() {
        let err =
            iopl(4 as UnsignedInt).expect_err("iopl(4) should not succeed");

        assert!(matches!(err, IoplError::Einval | IoplError::Enosys));
    }

    #[test]
    fn test_iopl_zero_level() {
        match iopl(0 as UnsignedInt) {
            Ok(()) => {}
            Err(IoplError::Enosys) => {}
            Err(other) => panic!("unexpected iopl(0) error: {other:?}"),
        }
    }

    #[test]
    fn test_iopl_error_mapping() {
        assert_eq!(IoplError::from_errno(Errno::Einval), IoplError::Einval);
        assert_eq!(IoplError::from_errno(Errno::Eperm), IoplError::Eperm);
        assert_eq!(IoplError::from_errno(Errno::Enosys), IoplError::Enosys);
        assert_eq!(
            IoplError::from_errno(Errno::Eio),
            IoplError::Other(Errno::Eio)
        );
    }
}
