use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`idle`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdleError {
    /// `ENOSYS`.
    Enosys,
    /// Another errno reported by the raw ABI.
    Other(Errno),
}

impl IdleError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Call the x86 `idle` syscall slot.
///
/// This safe wrapper keeps the raw no-argument syscall shape and maps the raw
/// return into `Result<(), IdleError>`.
///
/// `Ok(())` means the kernel reported success for the syscall slot.
///
/// See [`sys::idle`] for kernel behavior, historical notes, reachable errors,
/// and source references.
///
/// # Errors
/// - [`IdleError::Enosys`]: the current x86 syscall table has no implemented
///   `idle` entry.
/// - [`IdleError::Other`]: any other syscall error reported by the raw ABI.
pub fn idle() -> Result<(), IdleError> {
    let ret = sys::idle();

    unit_from_ret(ret as isize, IdleError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{IdleError, idle};
    use crate::{Errno, sys};

    #[test]
    fn test_idle_matches_raw() {
        let wrapped = idle();
        let raw = sys::idle();

        assert_eq!(wrapped, Err(IdleError::Enosys));
        assert_eq!(raw, -38);
    }

    #[test]
    fn test_idle_error_mapping() {
        assert_eq!(IdleError::from_errno(Errno::Enosys), IdleError::Enosys);
        assert_eq!(
            IdleError::from_errno(Errno::Einval),
            IdleError::Other(Errno::Einval)
        );
    }
}
