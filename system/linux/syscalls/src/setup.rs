use celer_system_linux_ctypes::Void;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`setup`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetupError {
    Eperm,
    Other(Errno),
}

impl SetupError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Run the historical Linux 1.0 bootstrap-only `setup` syscall.
///
/// This safe wrapper maps the raw Linux 1.0 return into
/// `Result<(), SetupError>`.
///
/// See [`sys::linux_1_0::setup`] for kernel behavior, reachable errors, and
/// source references.
///
/// # Errors
/// - [`SetupError::Eperm`]: Linux 1.0 returned literal `-1` when the one-shot
///   guard rejected a repeated call.
/// - [`SetupError::Other`]: any other errno-shaped raw return.
#[cfg(target_arch = "x86")]
pub fn setup(bios: *mut Void) -> Result<(), SetupError> {
    let ret = sys::linux_1_0::setup(bios);
    unit_from_ret(ret as isize, SetupError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::SetupError;

    #[test]
    fn test_setup_error_mapping() {
        assert_eq!(SetupError::from_errno(Errno::Eperm), SetupError::Eperm);
        assert_eq!(
            SetupError::from_errno(Errno::Eio),
            SetupError::Other(Errno::Eio)
        );
    }
}
