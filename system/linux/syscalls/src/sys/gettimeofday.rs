use celer_system_linux_ctypes::{Int, Timeval, Timezone};

use crate::arch::current::{Sysno, syscall2};

/// Get the current time of day and, optionally, the kernel timezone state.
///
/// # Safety
/// - `tv`, when non-null, must be valid to write one `Timeval` value for the
///   duration of the syscall.
/// - `tz`, when non-null, must be valid to write one `Timezone` value for the
///   duration of the syscall.
/// - `tv` and `tz`, when non-null, must not alias live Rust references or
///   other memory that would violate Rust's aliasing rules while the kernel
///   may write through those pointers.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 1.0 began returning `EFAULT` for invalid non-null
///   output pointers after `verify_area` started reporting failures.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - If `tv` is non-null, the kernel writes the current seconds since the Unix
///   epoch to `tv_sec` and the microsecond component within the current second
///   to `tv_usec`.
/// - If `tz` is non-null, the kernel writes the kernel timezone fields
///   `tz_minuteswest` and `tz_dsttime`.
/// - Either output pointer may be null independently, and the syscall still
///   succeeds.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EFAULT`: on Linux 1.0, a non-null `tv` or `tz` pointer is not writable
///   for one output structure.
/// - `EFAULT`: on current kernels, a non-null `tv` or `tz` pointer cannot be
///   written back to user memory.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/gettimeofday.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v7.0#n140)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n88)
/// - Stable x86_64 table: [v7.0 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n108)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n453)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n140)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n88)
/// - LTS x86_64 table: [v6.18.18 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v6.18.18#n108)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n453)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/time.c?h=1.0#n185)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.12#n440)
pub unsafe fn gettimeofday(tv: *mut Timeval, tz: *mut Timezone) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Gettimeofday, tv.addr() as isize, tz.addr() as isize)
            as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{Timeval, Timezone};

    use crate::arch::current::Sysno;

    use super::gettimeofday;

    #[test]
    fn test_gettimeofday_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 78;
        #[cfg(target_arch = "aarch64")]
        let expected = 169;
        #[cfg(target_arch = "x86_64")]
        let expected = 96;

        assert_eq!(Sysno::Gettimeofday as isize, expected);
    }

    #[test]
    fn test_gettimeofday_both_outputs() {
        let mut tv = Timeval {
            tv_sec: -1,
            tv_usec: -1,
        };
        let mut tz = Timezone {
            tz_minuteswest: -1,
            tz_dsttime: -1,
        };

        // SAFETY: `tv` and `tz` are valid writable output buffers.
        let rc = unsafe { gettimeofday(&raw mut tv, &raw mut tz) };

        assert_eq!(rc, 0, "gettimeofday failed: {rc}");
        assert!(tv.tv_sec > 0, "tv_sec should be a positive epoch value");
        assert!(
            (0..1_000_000).contains(&tv.tv_usec),
            "tv_usec should be in [0, 1_000_000), got {}",
            tv.tv_usec
        );
        assert_ne!(tz.tz_minuteswest, -1);
        assert_ne!(tz.tz_dsttime, -1);
    }

    #[test]
    fn test_gettimeofday_tv_null() {
        let mut tz = Timezone {
            tz_minuteswest: -1,
            tz_dsttime: -1,
        };

        // SAFETY: `tz` is a valid writable output buffer and `tv` may be
        // null.
        let rc = unsafe { gettimeofday(core::ptr::null_mut(), &raw mut tz) };

        assert_eq!(rc, 0, "gettimeofday failed: {rc}");
        assert_ne!(tz.tz_minuteswest, -1);
        assert_ne!(tz.tz_dsttime, -1);
    }

    #[test]
    fn test_gettimeofday_tz_null() {
        let mut tv = Timeval {
            tv_sec: -1,
            tv_usec: -1,
        };

        // SAFETY: `tv` is a valid writable output buffer and `tz` may be
        // null.
        let rc = unsafe { gettimeofday(&raw mut tv, core::ptr::null_mut()) };

        assert_eq!(rc, 0, "gettimeofday failed: {rc}");
        assert!(tv.tv_sec > 0, "tv_sec should be a positive epoch value");
        assert!(
            (0..1_000_000).contains(&tv.tv_usec),
            "tv_usec should be in [0, 1_000_000), got {}",
            tv.tv_usec
        );
    }

    #[test]
    fn test_gettimeofday_both_null() {
        // SAFETY: both pointers may be null for this syscall.
        let rc = unsafe {
            gettimeofday(core::ptr::null_mut(), core::ptr::null_mut())
        };

        assert_eq!(rc, 0, "gettimeofday failed: {rc}");
    }

    #[test]
    fn test_gettimeofday_bad_tv_pointer_returns_efault() {
        let rc = unsafe {
            gettimeofday(ptr::without_provenance_mut(1), ptr::null_mut())
        };

        assert_eq!(rc, -14, "expected EFAULT for bad tv pointer, got {rc}");
    }

    #[test]
    fn test_gettimeofday_bad_tz_pointer_returns_efault() {
        let rc = unsafe {
            gettimeofday(ptr::null_mut(), ptr::without_provenance_mut(1))
        };

        assert_eq!(rc, -14, "expected EFAULT for bad tz pointer, got {rc}");
    }
}
