use celer_system_linux_ctypes::UnsignedLong;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`brk`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BrkError {
    /// `EINTR`.
    Eintr,
    /// Another errno returned by the raw syscall.
    Other(Errno),
    /// The kernel returned a different current program break than requested.
    Rejected { current: UnsignedLong },
}

impl BrkError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eintr => Self::Eintr,
            errno => Self::Other(errno),
        }
    }
}

/// Set the program break to `addr` and return the resulting break on success.
///
/// This wrapper maps errno-shaped raw returns into `BrkError` and treats a
/// non-errno return equal to `addr` as success. A different non-errno return is
/// reported as [`BrkError::Rejected`].
///
/// See [`sys::brk`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// - `addr` must not move the program break in a way that invalidates Rust or
///   allocator-managed memory assumptions in the current process.
///
/// # Errors
/// - [`BrkError::Eintr`]: the raw syscall returned `EINTR`.
/// - [`BrkError::Other`]: the raw syscall returned another errno-shaped value.
/// - [`BrkError::Rejected`]: the returned current break differed from `addr`.
pub unsafe fn brk(addr: UnsignedLong) -> Result<UnsignedLong, BrkError> {
    // SAFETY: the caller must uphold the process-wide allocator and memory-map
    // invariants required when changing the program break.
    let ret = unsafe { sys::brk(addr) };

    result_from_ret(
        ret as isize,
        |_| {
            let current = ret;
            if current == addr {
                Ok(current)
            } else {
                Err(BrkError::Rejected { current })
            }
        },
        BrkError::from_errno,
    )
    .and_then(core::convert::identity)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UnsignedLong;

    use crate::{Errno, sys::test_support::process_global_state_guard};

    use super::{BrkError, brk};

    #[test]
    fn test_brk_roundtrip() {
        let _guard = process_global_state_guard();

        let current = unsafe { brk(0 as UnsignedLong) };
        let current = match current {
            Ok(value) => value,
            Err(BrkError::Rejected { current }) => current,
            Err(BrkError::Eintr) => panic!("brk(0) was interrupted"),
            Err(BrkError::Other(errno)) => {
                panic!("brk(0) returned unexpected errno: {errno:?}")
            }
        };
        assert_ne!(current, 0);

        let same = unsafe { brk(current) };
        assert_eq!(same, Ok(current));
    }

    #[test]
    fn test_brk_error_mapping() {
        assert_eq!(BrkError::from_errno(Errno::Eintr), BrkError::Eintr);
        assert_eq!(
            BrkError::from_errno(Errno::Enomem),
            BrkError::Other(Errno::Enomem)
        );
    }
}
