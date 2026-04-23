use celer_system_linux_ctypes::UnsignedInt;

use crate::arch::current::{Sysno, syscall1};

/// Arm the process alarm timer and return the remaining seconds on any prior alarm.
///
/// This is the Linux `alarm` syscall entry point.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Availability: common Linux syscall
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Arms `ITIMER_REAL` for `seconds` seconds.
/// - Cancels the previous alarm if one was pending.
/// - Returns the number of whole seconds remaining on the previous alarm.
///
/// # Errors
/// - Never fails (the return value reports the previous alarm instead of errno).
///
/// # Safety
/// - The caller must accept the side effect of changing the process alarm timer.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/alarm.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/itimer.c?h=v6.19#n325)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/itimer.c?h=v6.18.18#n325)
///
/// # Historical References
/// - Linux 1.0 returned the previous alarm time using floor semantics for
///   fractional remainders, while current kernels round up in some cases
///   before returning the previous remaining seconds.
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n342)
pub fn alarm(seconds: UnsignedInt) -> UnsignedInt {
    // SAFETY: `alarm` is a safe syscall with no pointer arguments.
    unsafe { syscall1(Sysno::Alarm, seconds as isize) as UnsignedInt }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::alarm;

    #[test]
    fn test_alarm_roundtrip() {
        let old = alarm(2);
        let cleared = alarm(0);

        assert!(old <= 2);
        assert!(cleared <= 2);
    }
}
