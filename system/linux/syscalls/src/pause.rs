use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`pause`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PauseError {
    /// `EINTR`.
    Eintr,
    /// Another errno returned by signal restart handling.
    Other(Errno),
}

impl PauseError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eintr => Self::Eintr,
            errno => Self::Other(errno),
        }
    }
}

/// Suspend the calling thread until a signal is pending for delivery.
///
/// This safe wrapper maps the raw syscall return value into
/// `Result<(), PauseError>`. The raw `pause` syscall has no arguments.
///
/// See [`sys::pause`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`PauseError::Eintr`]: a signal interrupted the sleep.
/// - [`PauseError::Other`]: another errno from signal restart handling.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn pause() -> Result<(), PauseError> {
    let ret = sys::pause();

    unit_from_ret(ret as isize, PauseError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::PauseError;

    #[test]
    fn test_pause_error_mapping() {
        assert_eq!(PauseError::from_errno(Errno::Eintr), PauseError::Eintr);
        assert_eq!(
            PauseError::from_errno(Errno::Eio),
            PauseError::Other(Errno::Eio)
        );
    }
}
