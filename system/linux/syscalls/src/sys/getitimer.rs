use celer_system_linux_ctypes::{Int, Itimerval};

use crate::arch::current::{Sysno, syscall2};

/// Read the current state of one process interval timer.
///
/// Linux 1.0 accepts only `ITIMER_REAL`, `ITIMER_VIRTUAL`, and
/// `ITIMER_PROF` at the syscall entry point.
///
/// # Safety
/// - `value`, when non-null, must be valid to write one [`Itimerval`] value
///   for the duration of the syscall.
/// - If `value` is non-null, the pointed-to memory must not violate Rust's
///   aliasing rules while the kernel may write through it.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: current kernels use different internal timekeeping
///   helpers before copying the same user-visible structure
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - When `which` is `ITIMER_REAL`, writes the current real-time timer state to
///   `value`.
/// - When `which` is `ITIMER_VIRTUAL`, writes the current virtual CPU timer
///   state to `value`.
/// - When `which` is `ITIMER_PROF`, writes the current profiling timer state
///   to `value`.
/// - On success, writes both `it_interval` and `it_value` to `value` and
///   returns `0`.
/// - Linux 1.0 derives the returned `timeval` fields from jiffies, so
///   `tv_usec` is quantized to the kernel tick size.
///
/// # Errors
/// - `EINVAL`: `which` is not `ITIMER_REAL`, `ITIMER_VIRTUAL`, or
///   `ITIMER_PROF`.
/// - `EFAULT`: `value` is null or is not writable for one [`Itimerval`].
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getitimer.2.html)
/// - Stable: [v7.0-rc7](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/itimer.c?h=v7.0-rc7#n113)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/itimer.c?h=v6.18.18#n113)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/itimer.c?h=1.0#n56)
///
/// # Historical References
/// - Linux 1.0 `struct itimerval`:
///   [include/linux/time.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/time.h?h=1.0#n30)
pub unsafe fn getitimer(which: Int, value: *mut Itimerval) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Getitimer, which as isize, value.addr() as isize) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{
        ITIMER_PROF, ITIMER_REAL, ITIMER_VIRTUAL, Itimerval, Timeval,
    };

    use super::getitimer;
    use crate::arch::current::{Sysno, syscall2};

    fn sentinel_itimerval() -> Itimerval {
        Itimerval {
            it_interval: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
            it_value: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
        }
    }

    #[test]
    fn test_getitimer_layout() {
        assert_eq!(ITIMER_REAL, 0);
        assert_eq!(ITIMER_VIRTUAL, 1);
        assert_eq!(ITIMER_PROF, 2);
        assert_eq!(core::mem::size_of::<Itimerval>(), 16);
        assert_eq!(core::mem::align_of::<Itimerval>(), 4);
        assert_eq!(core::mem::offset_of!(Itimerval, it_interval), 0);
        assert_eq!(core::mem::offset_of!(Itimerval, it_value), 8);
    }

    #[test]
    fn test_getitimer_sysno() {
        assert_eq!(Sysno::Getitimer as isize, 105);
    }

    #[test]
    fn test_getitimer_real_succeeds() {
        let mut timer = sentinel_itimerval();

        // SAFETY: `timer` is writable for one `Itimerval`.
        let rc = unsafe { getitimer(ITIMER_REAL, &raw mut timer) };

        assert_eq!(rc, 0, "getitimer(ITIMER_REAL) failed: {rc}");
        assert_ne!(timer.it_interval.tv_sec, -1);
        assert_ne!(timer.it_interval.tv_usec, -1);
        assert_ne!(timer.it_value.tv_sec, -1);
        assert_ne!(timer.it_value.tv_usec, -1);
    }

    #[test]
    fn test_getitimer_virtual_succeeds() {
        let mut timer = sentinel_itimerval();

        // SAFETY: `timer` is writable for one `Itimerval`.
        let rc = unsafe { getitimer(ITIMER_VIRTUAL, &raw mut timer) };

        assert_eq!(rc, 0, "getitimer(ITIMER_VIRTUAL) failed: {rc}");
        assert_ne!(timer.it_interval.tv_sec, -1);
        assert_ne!(timer.it_interval.tv_usec, -1);
        assert_ne!(timer.it_value.tv_sec, -1);
        assert_ne!(timer.it_value.tv_usec, -1);
    }

    #[test]
    fn test_getitimer_prof_succeeds() {
        let mut timer = sentinel_itimerval();

        // SAFETY: `timer` is writable for one `Itimerval`.
        let rc = unsafe { getitimer(ITIMER_PROF, &raw mut timer) };

        assert_eq!(rc, 0, "getitimer(ITIMER_PROF) failed: {rc}");
        assert_ne!(timer.it_interval.tv_sec, -1);
        assert_ne!(timer.it_interval.tv_usec, -1);
        assert_ne!(timer.it_value.tv_sec, -1);
        assert_ne!(timer.it_value.tv_usec, -1);
    }

    #[test]
    fn test_getitimer_rejects_invalid_which() {
        let mut timer = sentinel_itimerval();

        // SAFETY: `timer` is writable for one `Itimerval`.
        let rc = unsafe { getitimer(3, &raw mut timer) };

        assert_eq!(rc, -22, "expected EINVAL for invalid which, got {rc}");
        assert_eq!(timer, sentinel_itimerval());
    }

    #[test]
    fn test_getitimer_faults_on_null_pointer() {
        // SAFETY: this intentionally passes a null pointer to verify the
        // syscall reports `EFAULT`.
        let rc = unsafe { getitimer(ITIMER_REAL, core::ptr::null_mut()) };

        assert_eq!(rc, -14, "expected EFAULT for null timer pointer, got {rc}");
    }

    #[test]
    fn test_getitimer_faults_on_invalid_pointer() {
        // SAFETY: this intentionally passes an invalid pointer to verify the
        // syscall reports `EFAULT`.
        let bad_value =
            core::ptr::without_provenance_mut::<Itimerval>(usize::MAX);
        let rc = unsafe { getitimer(ITIMER_REAL, bad_value) };

        assert_eq!(rc, -14, "expected EFAULT for bad timer pointer, got {rc}");
    }

    #[test]
    fn test_getitimer_matches_raw_syscall_on_invalid_which() {
        let mut wrapped_timer = sentinel_itimerval();
        // SAFETY: `wrapped_timer` is writable for one `Itimerval`.
        let wrapped = unsafe { getitimer(3, &raw mut wrapped_timer) };

        let mut raw_timer = sentinel_itimerval();
        // SAFETY: `raw_timer` is writable for one `Itimerval`.
        let raw = unsafe {
            syscall2(Sysno::Getitimer, 3, (&raw mut raw_timer).addr() as isize)
        };

        assert_eq!(wrapped, -22, "wrapped getitimer should return EINVAL");
        assert_eq!(raw, -22, "raw getitimer should return EINVAL");
        assert_eq!(
            wrapped_timer, raw_timer,
            "EINVAL should leave the destination buffer unchanged"
        );
    }
}
