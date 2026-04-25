use celer_system_linux_ctypes::{Int, Long, UnsignedLong};

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

pub use sys::{
    SYS_ACCEPT, SYS_BIND, SYS_CONNECT, SYS_GETPEERNAME, SYS_GETSOCKNAME,
    SYS_GETSOCKOPT, SYS_LISTEN, SYS_RECV, SYS_RECVFROM, SYS_SEND, SYS_SENDTO,
    SYS_SETSOCKOPT, SYS_SHUTDOWN, SYS_SOCKET, SYS_SOCKETPAIR,
};

/// Errors returned by [`socketcall`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SocketcallError {
    Efault,
    Einval,
    Other(Errno),
}

impl SocketcallError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Dispatch a socket operation through the historical `socketcall` multiplexor.
///
/// This wrapper takes the packed `unsigned long` argument vector as a shared
/// slice and maps the raw return into `Result<Long, SocketcallError>`. The
/// required slice length and the meaning of pointers stored inside the slice
/// remain selected by `call`.
///
/// See [`sys::socketcall`] for kernel behavior, subcall selectors, reachable
/// errors, and source references.
///
/// # Safety
/// The `args` slice must contain at least the number of packed words required
/// by `call`, and any userspace pointers encoded inside it must satisfy the
/// selected socket operation's memory contract for the duration of the syscall.
///
/// # Errors
/// - [`SocketcallError::Efault`]: the kernel could not read the packed
///   argument vector or a subcall pointer.
/// - [`SocketcallError::Einval`]: `call` is not a recognized selector, or a
///   subcall rejected an invalid argument.
/// - [`SocketcallError::Other`]: any delegated socket helper error.
#[cfg(target_arch = "x86")]
pub unsafe fn socketcall(
    call: Int,
    args: &[UnsignedLong],
) -> Result<Long, SocketcallError> {
    // SAFETY: the caller guarantees the slice is long enough for `call` and
    // any encoded pointers satisfy the selected subcall's contract.
    let ret = unsafe { sys::socketcall(call, args.as_ptr()) };
    result_from_ret(
        ret as isize,
        |ret| ret as Long,
        SocketcallError::from_errno,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, UnsignedLong};

    use super::{SYS_SOCKETPAIR, SocketcallError, socketcall};
    use crate::Errno;
    use crate::sys::close;

    const AF_UNIX: Int = 1;
    const SOCK_STREAM: Int = 1;

    #[test]
    fn test_socketcall_invalid_selector_returns_einval() {
        let args = [0 as UnsignedLong; 6];

        let err = unsafe { socketcall(0, &args) }.unwrap_err();

        assert_eq!(err, SocketcallError::Einval);
    }

    #[test]
    fn test_socketcall_socketpair_ok() {
        let mut fds = [-1 as Int; 2];
        let args = [
            AF_UNIX as UnsignedLong,
            SOCK_STREAM as UnsignedLong,
            0 as UnsignedLong,
            (&raw mut fds).addr() as UnsignedLong,
        ];

        let result = unsafe { socketcall(SYS_SOCKETPAIR, &args) };

        assert_eq!(result, Ok(0));
        assert!(fds[0] >= 0, "invalid first socket fd: {}", fds[0]);
        assert!(fds[1] >= 0, "invalid second socket fd: {}", fds[1]);
        assert_ne!(fds[0], fds[1]);
        assert_eq!(close(fds[0]), 0);
        assert_eq!(close(fds[1]), 0);
    }

    #[test]
    fn test_socketcall_error_mapping() {
        assert_eq!(
            SocketcallError::from_errno(Errno::Efault),
            SocketcallError::Efault
        );
        assert_eq!(
            SocketcallError::from_errno(Errno::Einval),
            SocketcallError::Einval
        );
        assert_eq!(
            SocketcallError::from_errno(Errno::Eacces),
            SocketcallError::Other(Errno::Eacces)
        );
    }
}
