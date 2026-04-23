use celer_system_linux_ctypes::UnsignedLong;

use crate::arch::current::{Sysno, syscall1};

/// Adjust the program break to `addr`.
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
/// # Behavior
/// - On success, returns the resulting break value.
/// - If the requested break stays within the same page, the kernel updates the
///   in-memory break value and returns the requested address.
/// - If the request cannot be satisfied, the kernel usually returns the
///   previous break value.
/// - Interrupted lock acquisition can surface as raw `-EINTR`.
/// - The raw return value is an address value, so callers must compare it
///   against the requested break instead of assuming negative means failure.
///
/// # Errors
/// - The raw return value is address-valued: most unsatisfied requests return
///   the previous break value rather than a conventional errno.
/// - Interrupted mmap-lock acquisition can return raw `-EINTR`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/brk.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/mmap.c?h=v6.19#n115)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/mmap.c?h=v6.18.18#n115)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n168)
pub unsafe fn brk(addr: UnsignedLong) -> UnsignedLong {
    // SAFETY: the caller must uphold the process-wide allocator and memory-map
    // invariants required when changing the program break.
    unsafe { syscall1(Sysno::Brk, addr as isize) as UnsignedLong }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UnsignedLong;

    use super::brk;

    #[test]
    fn test_brk_roundtrip() {
        let current = unsafe { brk(0 as UnsignedLong) };
        assert_ne!(current, 0);

        let same = unsafe { brk(current) };
        assert_eq!(same, current);
    }
}
