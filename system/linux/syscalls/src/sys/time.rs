use celer_system_linux_ctypes::TimeT;

use crate::arch::current::{Sysno, syscall1};

/// Return the current calendar time in seconds since the Unix epoch.
///
/// If `tloc` is non-null, the kernel also stores the same value at that
/// address before returning.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Returns the current real time in whole seconds since the Unix epoch.
/// - If `tloc` is non-null, stores the same value through `tloc`.
/// - On success, the return value is the same seconds value that was optionally
///   written through `tloc`.
///
/// # Errors
/// - `EFAULT`: `tloc`, when non-null, does not point to writable memory.
///
/// # Safety
/// - `tloc`, when non-null, must be valid to write a single `TimeT` value for
///   the duration of the syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/time.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v6.19#n62)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n62)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/time/time.c?h=1.0#n26)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/time.c?h=0.10#n1)
pub unsafe fn time(tloc: *mut TimeT) -> TimeT {
    // SAFETY: `tloc` is passed through exactly as provided by the caller.
    unsafe { syscall1(Sysno::Time, tloc.addr() as isize) as TimeT }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::TimeT;

    use super::time;

    #[test]
    fn test_time_null() {
        // SAFETY: passing a null pointer is explicitly supported by `time`.
        let now = unsafe { time(core::ptr::null_mut()) };

        assert!(now > 0, "time should return a positive epoch value");
    }

    #[test]
    fn test_time_write_through_pointer() {
        let mut stored: TimeT = 0;
        // SAFETY: `stored` is a valid writable `Long` for the duration of the call.
        let now = unsafe { time(&raw mut stored) };

        assert_eq!(now, stored);
        assert!(now > 0, "time should return a positive epoch value");
    }
}
