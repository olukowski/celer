use celer_system_linux_ctypes::PidT;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`fork`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ForkError {
    /// `EAGAIN`.
    Eagain,
    /// `EINTR`.
    Eintr,
    /// `ENOMEM`.
    Enomem,
    /// Another errno returned by delegated security, cgroup, or namespace work.
    Other(Errno),
}

impl ForkError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eagain => Self::Eagain,
            Errno::Eintr => Self::Eintr,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Create a child process.
///
/// This safe wrapper maps the raw `fork(2)` return value into
/// `Result<PidT, ForkError>`.
///
/// On success, returns `Ok(0)` in the child process and `Ok(pid)` in the
/// parent process.
///
/// See [`sys::fork`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ForkError::Eagain`]: the kernel refused to create another task because
///   process or thread limits were hit.
/// - [`ForkError::Eintr`]: a fatal signal interrupted `fork` after the child
///   task had been prepared but before it became visible.
/// - [`ForkError::Enomem`]: the kernel could not allocate the child task or
///   related resources.
/// - [`ForkError::Other`]: another delegated security, cgroup, or namespace
///   error.
pub fn fork() -> Result<PidT, ForkError> {
    let ret = sys::fork();

    result_from_ret(ret as isize, |ret| ret as PidT, ForkError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::{ForkError, fork};
    use crate::{Errno, exit};

    #[test]
    fn test_fork_ok() {
        let pid = fork().expect("fork should succeed");

        if pid == 0 {
            exit(0);
        }

        let mut status: Int = 0;
        let waited = unsafe { libc::waitpid(pid, &mut status, 0) };

        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }

    #[test]
    fn test_fork_error_mapping() {
        assert_eq!(ForkError::from_errno(Errno::Eagain), ForkError::Eagain);
        assert_eq!(ForkError::from_errno(Errno::Eintr), ForkError::Eintr);
        assert_eq!(ForkError::from_errno(Errno::Enomem), ForkError::Enomem);
        assert_eq!(
            ForkError::from_errno(Errno::Eperm),
            ForkError::Other(Errno::Eperm)
        );
    }
}
