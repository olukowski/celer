use celer_system_linux_ctypes::{Int, TimeT};

use crate::arch::current::{Sysno, syscall1};

/// Set the system clock to a whole-second Unix timestamp.
///
/// This is the legacy `stime` entry point on x86 Linux.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Availability: x86 Linux only for now
///
/// # Required Privileges
/// - Requires permission to set the system time.
///
/// # Behavior
/// - Reads a whole-second timestamp from `tptr`.
/// - Zeroes the nanosecond field before handing the timestamp to the kernel
///   time-setting path.
/// - Calls the kernel time-setting helper and returns the syscall result.
/// - A return value of `0` means the syscall body completed, not necessarily
///   that the requested clock value was accepted by every internal helper.
///
/// # Errors
/// - `EFAULT`: `tptr` does not point to readable memory.
/// - `EPERM`: the caller is not permitted to set the system time.
///
/// # Safety
/// - `tptr` must be valid to read a single `TimeT` value for the duration of
///   the syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stime.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v6.19#n81)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n81)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n40)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n148)
pub unsafe fn stime(tptr: *const TimeT) -> Int {
    // SAFETY: `tptr` is passed through exactly as provided by the caller.
    unsafe { syscall1(Sysno::Stime, tptr.addr() as isize) as Int }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::TimeT;

    use super::stime;

    #[test]
    fn test_stime_null_pointer_faults() {
        // SAFETY: passing a null pointer is permitted by the kernel ABI and
        // should fail before any attempt to set the clock.
        let ret = unsafe { stime(core::ptr::null()) };

        assert_eq!(ret, -14);
    }
}
