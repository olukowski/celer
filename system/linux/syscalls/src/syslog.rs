use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Char, Int};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`syslog`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SyslogError {
    Eperm,
    Einval,
    Efault,
    Eintr,
    Enomem,
    Other(Errno),
}

impl SyslogError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            Errno::Eintr => Self::Eintr,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Read from or control the kernel log buffer.
///
/// This safe wrapper keeps the raw multiplexed `type_` command and `len`
/// integer, replaces the raw output pointer with an optional initialized
/// capacity `buf`, and maps the raw return into `Result<Int, SyslogError>`.
///
/// When `buf` is `Some`, `len` must be nonnegative and no larger than the
/// slice length. When `buf` is `None`, the raw syscall receives a null pointer.
///
/// On success, returns the raw command result: `0` for control commands, a
/// byte count for read commands, or a size/count value for query commands.
///
/// See [`sys::syslog`] for command semantics, required privileges, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SyslogError::Eperm`]: the selected command requires permission.
/// - [`SyslogError::Einval`]: the command or length is invalid, or `len`
///   exceeds the provided buffer.
/// - [`SyslogError::Efault`]: the kernel could not write the buffer.
/// - [`SyslogError::Eintr`]: a blocking read was interrupted.
/// - [`SyslogError::Enomem`]: temporary log-copy allocation failed.
/// - [`SyslogError::Other`]: delegated security-policy error.
pub fn syslog(
    type_: Int,
    buf: Option<&mut [MaybeUninit<u8>]>,
    len: Int,
) -> Result<Int, SyslogError> {
    let ptr = match buf {
        Some(buf) => {
            let len = usize::try_from(len).map_err(|_| SyslogError::Einval)?;
            if len > buf.len() {
                return Err(SyslogError::Einval);
            }
            buf.as_mut_ptr().cast::<Char>()
        }
        None => core::ptr::null_mut(),
    };

    // SAFETY: a non-null pointer is backed by `buf` for at least `len` bytes;
    // null is passed only when `buf` is `None`.
    let ret = unsafe { sys::syslog(type_, ptr, len) };
    result_from_ret(ret as isize, |ret| ret as Int, SyslogError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use crate::Errno;

    use super::{SyslogError, syslog};

    #[test]
    fn test_syslog_rejects_len_larger_than_buffer() {
        let mut buf = [MaybeUninit::uninit(); 4];

        assert_eq!(syslog(3, Some(&mut buf), 5), Err(SyslogError::Einval));
    }

    #[test]
    fn test_syslog_accepts_zero_len_buffer() {
        let mut buf = [];
        let result = syslog(3, Some(&mut buf), 0);

        assert!(matches!(result, Ok(0) | Err(SyslogError::Eperm)));
    }

    #[test]
    fn test_syslog_invalid_type_is_permission_or_invalid() {
        let err = syslog(11, None, 0).unwrap_err();
        assert!(matches!(err, SyslogError::Eperm | SyslogError::Einval));
    }

    #[test]
    fn test_syslog_error_mapping() {
        assert_eq!(SyslogError::from_errno(Errno::Eperm), SyslogError::Eperm);
        assert_eq!(SyslogError::from_errno(Errno::Einval), SyslogError::Einval);
        assert_eq!(SyslogError::from_errno(Errno::Efault), SyslogError::Efault);
        assert_eq!(SyslogError::from_errno(Errno::Eintr), SyslogError::Eintr);
        assert_eq!(SyslogError::from_errno(Errno::Enomem), SyslogError::Enomem);
        assert_eq!(
            SyslogError::from_errno(Errno::Eacces),
            SyslogError::Other(Errno::Eacces)
        );
    }
}
