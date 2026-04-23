use celer_system_linux_ctypes::{Int, TimeT};

use crate::arch::current::{Sysno, syscall1};

/// Set the system clock to a whole-second Unix timestamp.
///
/// This is the legacy `stime` entry point on x86 Linux.
///
/// # Safety
/// - `tptr` must be valid to read one [`TimeT`] for the duration of the
///   syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 checked permission before reading `tptr` and
///   did not validate the user pointer with `verify_area`; current kernels
///   first read `tptr` with `get_user` and return `EFAULT` when that read
///   fails
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
/// - Linux 1.0 performs the explicit permission check before reading `tptr`.
///
/// # Errors
/// - `EPERM`: the caller is not permitted to set the system time.
/// - Current kernels return `EFAULT` when `tptr` does not point to readable
///   memory.
/// - Linux 1.0 does not implement a source-backed recoverable `EFAULT` path
///   for invalid `tptr`; its syscall body only contains an explicit `EPERM`
///   return before the unchecked `get_fs_long(tptr)` load.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stime.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v6.19#n81)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n81)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n148)
pub unsafe fn stime(tptr: *const TimeT) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Stime, tptr.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::TimeT;

    use super::stime;

    #[test]
    fn test_stime_syscall_number() {
        assert_eq!(crate::arch::current::Sysno::Stime as isize, 25);
    }

    #[test]
    fn test_stime_invalid_time_is_rejected_or_permission_denied() {
        let invalid = -1 as TimeT;

        // SAFETY: `invalid` is readable for one `TimeT` for the duration of
        // the syscall.
        let ret = unsafe { stime(&raw const invalid) };

        assert!(
            matches!(ret, -1 | -22),
            "expected EPERM or EINVAL from stime(-1), got {ret}",
        );
    }
}
