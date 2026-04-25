use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`nice`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NiceError {
    /// `EPERM`.
    Eperm,
    /// Another errno returned by security hooks.
    Other(Errno),
}

impl NiceError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Change the current process nice value by a relative increment.
///
/// This safe wrapper keeps the raw increment argument and maps the raw syscall
/// return into `Result<(), NiceError>`.
///
/// On success, the kernel has applied the clamped nice adjustment.
///
/// See [`sys::nice`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`NiceError::Eperm`]: lowering the nice value is not permitted.
/// - [`NiceError::Other`]: a security hook denied the update with another
///   errno.
#[cfg(target_arch = "x86")]
pub fn nice(increment: Int) -> Result<(), NiceError> {
    let ret = sys::nice(increment);

    unit_from_ret(ret as isize, NiceError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use crate::Errno;

    use super::{NiceError, nice};

    #[test]
    fn test_nice_zero_increment_ok() {
        assert_eq!(nice(0 as Int), Ok(()));
    }

    #[test]
    fn test_nice_error_mapping() {
        assert_eq!(NiceError::from_errno(Errno::Eperm), NiceError::Eperm);
        assert_eq!(
            NiceError::from_errno(Errno::Eio),
            NiceError::Other(Errno::Eio)
        );
    }
}
