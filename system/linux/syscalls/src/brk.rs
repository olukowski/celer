use celer_system_linux_ctypes::UnsignedLong;

use crate::errno::Errno;
use crate::sys;

/// Errors returned by [`brk`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BrkError {
    Eintr,
    Rejected { current: UnsignedLong },
}

/// Set the program break to `addr` and return the resulting break on success.
///
/// The kernel returns the current program break value even when it declines
/// the request, so this wrapper treats a returned value equal to `addr` as
/// success and reports any other non-errno return as a semantic rejection.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern kernels can return raw `-EINTR` if interrupted
///   while acquiring the mmap write lock
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Safety
/// - `addr` must not move the program break in a way that invalidates Rust or
///   allocator-managed memory assumptions in the current process.
///
/// # Errors
/// - `EINTR`: the kernel was interrupted while acquiring the mmap write lock.
/// - `Rejected { .. }`: the kernel left the program break unchanged.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/brk.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/mmap.c?h=v6.19#n115)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/mmap.c?h=v6.18.18#n115)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n168)
pub unsafe fn brk(addr: UnsignedLong) -> Result<UnsignedLong, BrkError> {
    // SAFETY: the caller must uphold the process-wide allocator and memory-map
    // invariants required when changing the program break.
    let ret = unsafe { sys::brk(addr) };

    if Errno::is_errno(ret as isize) {
        return Err(BrkError::Eintr);
    }

    let current = ret;
    if current == addr {
        Ok(current)
    } else {
        Err(BrkError::Rejected { current })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UnsignedLong;

    use crate::sys::test_support::process_global_state_guard;

    use super::{BrkError, brk};

    #[test]
    fn test_brk_roundtrip() {
        let _guard = process_global_state_guard();

        let current = unsafe { brk(0 as UnsignedLong) };
        let current = match current {
            Ok(value) => value,
            Err(BrkError::Rejected { current }) => current,
            Err(BrkError::Eintr) => panic!("brk(0) was interrupted"),
        };
        assert_ne!(current, 0);

        let same = unsafe { brk(current) };
        assert_eq!(same, Ok(current));
    }
}
