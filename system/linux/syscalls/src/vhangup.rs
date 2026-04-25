use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`vhangup`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VhangupError {
    Eperm,
    Other(Errno),
}

impl VhangupError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            errno => Self::Other(errno),
        }
    }
}

/// Hang up the calling process's controlling terminal.
///
/// This safe wrapper maps the raw return value into
/// `Result<(), VhangupError>`.
///
/// See [`sys::vhangup`] for kernel behavior, privileges, reachable errors, and
/// source references.
///
/// # Errors
/// - [`VhangupError::Eperm`]: the caller lacks permission.
/// - [`VhangupError::Other`]: any other errno reported by the raw ABI.
pub fn vhangup() -> Result<(), VhangupError> {
    let ret = sys::vhangup();
    unit_from_ret(ret as isize, VhangupError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{VhangupError, vhangup};

    #[test]
    fn test_vhangup_reports_success_or_permission_error() {
        let ret = vhangup();
        assert!(matches!(ret, Ok(()) | Err(VhangupError::Eperm)));
    }

    #[test]
    fn test_vhangup_error_mapping() {
        assert_eq!(VhangupError::from_errno(Errno::Eperm), VhangupError::Eperm);
        assert_eq!(
            VhangupError::from_errno(Errno::Eio),
            VhangupError::Other(Errno::Eio)
        );
    }
}
