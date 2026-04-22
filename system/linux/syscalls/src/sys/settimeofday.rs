use core::arch::asm;

use celer_system_linux_ctypes::{Int, Timeval, Timezone};

use crate::arch::current::Sysno;

/// Set the system wall clock and/or legacy timezone state.
///
/// This is the historical x86 `settimeofday` entry point from Linux 1.0.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Availability: x86 Linux only for now
///
/// # Required Privileges
/// - Linux 1.0 required the superuser check in `sys_settimeofday`.
/// - Current kernels require permission to set system time, typically
///   `CAP_SYS_TIME`.
///
/// # Behavior
/// - If `tv` is non-null, the kernel reads `tv_sec` and `tv_usec` from user
///   memory and attempts to set the wall clock.
/// - If `tz` is non-null, the kernel updates the legacy kernel timezone state.
/// - `tv` and `tz` may both be null; the ABI still performs the privilege
///   check before returning.
/// - If `tz` is accepted, it is applied before any later failure from the
///   time-setting path, so an error does not guarantee that no timezone side
///   effects occurred.
/// - On the first successful call with `tz != NULL` and `tv == NULL`, Linux
///   1.0 and current kernels may warp the clock using the supplied timezone.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to set system time.
/// - Current kernels may return `EFAULT` if copying a non-null `tv` or `tz`
///   from user memory fails before the privilege check.
/// - Current kernels may return `EINVAL` if `tv_usec` is outside the accepted
///   range, `tz_minuteswest` lies outside +/-15 hours, or the requested
///   wall-clock change is rejected by the timekeeping path.
/// - Linux 1.0 has an explicit `EPERM` return in `sys_settimeofday`, but its
///   syscall body does not contain a corresponding explicit `EFAULT` or
///   `EINVAL` path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/settimeofday.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v7.0#n199)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n199)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/time.c?h=1.0#n240)
pub fn settimeofday(tv: *const Timeval, tz: *const Timezone) -> Int {
    let ret: isize;

    // SAFETY: `int 0x80` is the x86 Linux syscall instruction, and this
    // wrapper forwards the raw ABI arguments without dereferencing them in
    // Rust.
    unsafe {
        asm!(
            "int 0x80",
            inlateout("eax") Sysno::Settimeofday as usize => ret,
            in("ebx") tv.addr() as isize,
            in("ecx") tz.addr() as isize,
            options(nostack, preserves_flags),
        );
    }

    ret as Int
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::{Timeval, Timezone};

    use super::settimeofday;

    #[test]
    fn test_settimeofday_bad_tv_pointer_faults() {
        let tv = core::ptr::without_provenance::<Timeval>(1);

        let ret = settimeofday(tv, core::ptr::null());

        assert_eq!(ret, -14);
    }

    #[test]
    fn test_settimeofday_bad_tz_pointer_faults() {
        let tz = core::ptr::without_provenance::<Timezone>(1);

        let ret = settimeofday(core::ptr::null(), tz);

        assert_eq!(ret, -14);
    }

    #[test]
    fn test_settimeofday_invalid_tv_usec_rejected_before_privilege_check() {
        let tv = Timeval {
            tv_sec: 0,
            tv_usec: 1_000_001,
        };

        let ret = settimeofday(&tv, core::ptr::null());

        assert_eq!(ret, -22);
    }
}
