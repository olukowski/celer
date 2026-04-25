use celer_system_linux_ctypes::{FdSet, Int, Timeval};

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`select`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectError {
    Ebadf,
    Efault,
    Einval,
    Enomem,
    Eintr,
    Other(Errno),
}

impl SelectError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Eintr => Self::Eintr,
            other => Self::Other(other),
        }
    }
}

/// Wait for readiness changes on up to `nfds` file descriptors.
///
/// This wrapper makes the descriptor sets and timeout explicit nullable Rust
/// references. Non-null descriptor sets are read and then overwritten by the
/// kernel; a non-null timeout is read and then overwritten with the remaining
/// time.
///
/// `Ok(n)` is the total number of ready descriptor bits in the result sets.
///
/// See [`sys::select`] for kernel behavior, ABI differences, pointer sizing
/// requirements, reachable errors, and source references.
///
/// # Safety
/// The non-null descriptor-set references must cover the full kernel bitmap
/// size implied by the effective `nfds` on the running kernel. One [`FdSet`]
/// is sufficient only when that effective value does not exceed `1024`.
///
/// # Errors
/// - [`SelectError::Ebadf`]: one requested descriptor is not open.
/// - [`SelectError::Efault`]: the kernel could not access a non-null pointer.
/// - [`SelectError::Einval`]: `nfds` or `timeout` values are invalid.
/// - [`SelectError::Enomem`]: kernel wait-table allocation failed.
/// - [`SelectError::Eintr`]: an unblocked signal interrupted the wait.
/// - [`SelectError::Other`]: another syscall error reported by the raw ABI.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn select(
    nfds: Int,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    exceptfds: Option<&mut FdSet>,
    timeout: Option<&mut Timeval>,
) -> Result<usize, SelectError> {
    let readfds = readfds.map_or(core::ptr::null_mut(), |fds| fds);
    let writefds = writefds.map_or(core::ptr::null_mut(), |fds| fds);
    let exceptfds = exceptfds.map_or(core::ptr::null_mut(), |fds| fds);
    let timeout = timeout.map_or(core::ptr::null_mut(), |timeout| timeout);

    // SAFETY: forwarded from this wrapper's safety contract.
    let ret =
        unsafe { sys::select(nfds, readfds, writefds, exceptfds, timeout) };
    result_from_ret(ret as isize, |ret| ret as usize, SelectError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{FdSet, Int, Timeval, UnsignedLong};

    use crate::{Errno, sys};

    use super::{SelectError, select};

    fn empty_fd_set() -> FdSet {
        FdSet {
            #[cfg(target_arch = "x86")]
            fds_bits: [0; 32],
            #[cfg(target_arch = "x86_64")]
            fds_bits: [0; 16],
        }
    }

    fn set_fd(set: &mut FdSet, fd: Int) {
        let bits_per_long = UnsignedLong::BITS as usize;
        let fd = fd as usize;
        let word = fd / bits_per_long;
        let bit = fd % bits_per_long;

        set.fds_bits[word] |= (1 as UnsignedLong) << bit;
    }

    fn is_fd_set(set: &FdSet, fd: Int) -> bool {
        let bits_per_long = UnsignedLong::BITS as usize;
        let fd = fd as usize;
        let word = fd / bits_per_long;
        let bit = fd % bits_per_long;

        (set.fds_bits[word] & ((1 as UnsignedLong) << bit)) != 0
    }

    #[test]
    fn test_select_zero_timeout_with_null_sets() {
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        let ready =
            unsafe { select(0, None, None, None, Some(&mut timeout)) }.unwrap();

        assert_eq!(ready, 0);
        assert_eq!(timeout.tv_sec, 0);
        assert_eq!(timeout.tv_usec, 0);
    }

    #[test]
    fn test_select_pipe_read_end_ready() {
        let mut fds = [0 as Int; 2];
        let rc = unsafe { sys::pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");
        let msg = [b'x'];
        let written =
            unsafe { sys::write(fds[1] as _, msg.as_ptr().cast(), msg.len()) };
        assert_eq!(written, 1, "write failed: {written}");

        let mut readfds = empty_fd_set();
        set_fd(&mut readfds, fds[0]);
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        let ready = unsafe {
            select(
                fds[0] + 1,
                Some(&mut readfds),
                None,
                None,
                Some(&mut timeout),
            )
        }
        .unwrap();

        assert_eq!(ready, 1);
        assert!(is_fd_set(&readfds, fds[0]));
        assert_eq!(sys::close(fds[0]), 0);
        assert_eq!(sys::close(fds[1]), 0);
    }

    #[test]
    fn test_select_negative_nfds() {
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        assert_eq!(
            unsafe { select(-1, None, None, None, Some(&mut timeout)) },
            Err(SelectError::Einval)
        );
    }

    #[test]
    fn test_select_error_mapping() {
        assert_eq!(SelectError::from_errno(Errno::Ebadf), SelectError::Ebadf);
        assert_eq!(SelectError::from_errno(Errno::Efault), SelectError::Efault);
        assert_eq!(SelectError::from_errno(Errno::Einval), SelectError::Einval);
        assert_eq!(SelectError::from_errno(Errno::Enomem), SelectError::Enomem);
        assert_eq!(SelectError::from_errno(Errno::Eintr), SelectError::Eintr);
        assert_eq!(
            SelectError::from_errno(Errno::Eio),
            SelectError::Other(Errno::Eio)
        );
    }
}
