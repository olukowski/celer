use celer_system_linux_ctypes::{Int, Void};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`reboot`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RebootError {
    Eperm,
    Einval,
    Efault,
    Other(Errno),
}

impl RebootError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Reboot the system or toggle Ctrl-Alt-Del handling.
///
/// This wrapper preserves the raw magic values, command, and command-specific
/// argument pointer while mapping the raw return value into
/// `Result<(), RebootError>`.
///
/// See [`sys::reboot`] for kernel behavior, historical notes, reachable
/// errors, and source references.
///
/// # Safety
/// If `cmd` causes the kernel to read `arg`, `arg` must be valid to read the
/// command-specific user data for the duration of the syscall.
///
/// # Errors
/// - [`RebootError::Eperm`]: the caller lacks permission, or a delegated
///   suspend path reports permission or availability failure as `EPERM`.
/// - [`RebootError::Einval`]: the magic values or command are unsupported.
/// - [`RebootError::Efault`]: a command-specific user pointer could not be
///   read.
/// - [`RebootError::Other`]: another errno from command-specific reboot paths.
pub unsafe fn reboot(
    magic: Int,
    magic_too: Int,
    cmd: Int,
    arg: *const Void,
) -> Result<(), RebootError> {
    // SAFETY: forwarded from this wrapper's safety contract.
    let ret = unsafe { sys::reboot(magic, magic_too, cmd, arg) };
    unit_from_ret(ret as isize, RebootError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use crate::Errno;

    use super::{RebootError, reboot};

    #[test]
    fn test_reboot_bad_magic_is_rejected_or_permission_denied() {
        // SAFETY: this command does not cause the kernel to read `arg`.
        let err = unsafe { reboot(0, 0, 0, ptr::null()) }.unwrap_err();

        assert!(matches!(err, RebootError::Eperm | RebootError::Einval));
    }

    #[test]
    fn test_reboot_error_mapping() {
        assert_eq!(RebootError::from_errno(Errno::Eperm), RebootError::Eperm);
        assert_eq!(RebootError::from_errno(Errno::Einval), RebootError::Einval);
        assert_eq!(RebootError::from_errno(Errno::Efault), RebootError::Efault);
        assert_eq!(
            RebootError::from_errno(Errno::Ebusy),
            RebootError::Other(Errno::Ebusy)
        );
    }
}
