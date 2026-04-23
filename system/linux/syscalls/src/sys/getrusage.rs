use celer_system_linux_ctypes::{Int, Rusage};

use crate::arch::current::{Sysno, syscall2};

/// Return resource-usage accounting for the calling process or its waited-for
/// children.
///
/// Linux 1.0 only accepts [`RUSAGE_SELF`] and [`RUSAGE_CHILDREN`] at the
/// syscall entry point, even though the internal helper also supports the
/// `RUSAGE_BOTH` aggregate used by `wait4`.
///
/// # Kernel Support
/// - Introduced: Linux 0.95
/// - Behavior changes: Linux 1.0 accepted only [`RUSAGE_SELF`] and
///   [`RUSAGE_CHILDREN`]; current kernels also accept `RUSAGE_THREAD` and
///   populate additional accounting fields
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - When `who` is [`RUSAGE_SELF`], writes the caller's user time, system
///   time, minor faults, and major faults to `ru`.
/// - When `who` is [`RUSAGE_CHILDREN`], writes the same counters aggregated
///   across waited-for children to `ru`.
/// - Linux 1.0 zero-initialized the whole result structure before populating
///   those fields, so the remaining fields were returned as zero.
/// - On success, returns `0`.
///
/// # Errors
/// - `EINVAL`: `who` is not [`RUSAGE_SELF`] or [`RUSAGE_CHILDREN`] on the
///   Linux 1.0 syscall entry path.
/// - `EFAULT`: `ru` is not writable for one [`Rusage`] value.
///
/// # Safety
/// - `ru` must be valid to write one [`Rusage`] value for the duration of the
///   syscall.
/// - If `ru` is non-null, the pointed-to memory must not violate Rust's
///   aliasing rules while the kernel may write through it.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getrusage.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n1934)
/// - LTS: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v7.0#n1934)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n770)
///
/// # Historical References
/// - First appearance: [Linux 0.95](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.95#n446)
/// - Linux 1.0 `struct rusage`:
///   [include/linux/resource.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/resource.h?h=1.0#n19)
/// - Linux 1.0 `struct timeval`:
///   [include/linux/time.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/time.h?h=1.0#n4)
pub unsafe fn getrusage(who: Int, ru: *mut Rusage) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Getrusage, who as isize, ru.addr() as isize) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{
        RUSAGE_CHILDREN, RUSAGE_SELF, Rusage, Timeval,
    };

    use super::getrusage;
    use crate::arch::current::{Sysno, syscall2};

    fn zeroed_rusage() -> Rusage {
        Rusage {
            ru_utime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        }
    }

    #[test]
    fn test_getrusage_layout() {
        assert_eq!(core::mem::size_of::<Timeval>(), 8);
        assert_eq!(core::mem::align_of::<Timeval>(), 4);
        assert_eq!(core::mem::size_of::<Rusage>(), 72);
        assert_eq!(core::mem::align_of::<Rusage>(), 4);
        assert_eq!(core::mem::offset_of!(Rusage, ru_utime), 0);
        assert_eq!(core::mem::offset_of!(Rusage, ru_stime), 8);
        assert_eq!(core::mem::offset_of!(Rusage, ru_maxrss), 16);
        assert_eq!(core::mem::offset_of!(Rusage, ru_minflt), 32);
        assert_eq!(core::mem::offset_of!(Rusage, ru_majflt), 36);
        assert_eq!(core::mem::offset_of!(Rusage, ru_nivcsw), 68);
    }

    #[test]
    fn test_getrusage_self_succeeds() {
        let mut usage = zeroed_rusage();

        // SAFETY: `usage` is writable for one `Rusage`.
        let rc = unsafe { getrusage(RUSAGE_SELF, &raw mut usage) };

        assert_eq!(rc, 0, "getrusage(RUSAGE_SELF) failed: {rc}");
    }

    #[test]
    fn test_getrusage_children_succeeds() {
        let mut usage = zeroed_rusage();

        // SAFETY: `usage` is writable for one `Rusage`.
        let rc = unsafe { getrusage(RUSAGE_CHILDREN, &raw mut usage) };

        assert_eq!(rc, 0, "getrusage(RUSAGE_CHILDREN) failed: {rc}");
    }

    #[test]
    fn test_getrusage_rejects_invalid_who() {
        let mut usage = zeroed_rusage();

        // SAFETY: `usage` is writable for one `Rusage`.
        let rc = unsafe { getrusage(-2, &raw mut usage) };

        assert_eq!(rc, -22, "expected EINVAL for invalid who, got {rc}");
    }

    #[test]
    fn test_getrusage_faults_on_invalid_pointer() {
        // SAFETY: this intentionally passes an invalid pointer to verify the
        // syscall reports `EFAULT`.
        let rc = unsafe { getrusage(RUSAGE_SELF, usize::MAX as *mut Rusage) };

        assert_eq!(rc, -14, "expected EFAULT for bad rusage pointer, got {rc}");
    }

    #[test]
    fn test_getrusage_matches_raw_syscall_error_path() {
        let mut wrapped_usage = zeroed_rusage();
        // SAFETY: `wrapped_usage` is writable for one `Rusage`.
        let wrapped = unsafe { getrusage(-2, &raw mut wrapped_usage) };

        let mut raw_usage = zeroed_rusage();
        // SAFETY: `raw_usage` is writable for one `Rusage`.
        let raw = unsafe {
            syscall2(Sysno::Getrusage, -2, (&raw mut raw_usage).addr() as isize)
        };
        assert_eq!(wrapped, -22, "wrapped getrusage should return EINVAL");
        assert_eq!(raw, -22, "raw getrusage should return EINVAL");
        assert_eq!(
            wrapped_usage, raw_usage,
            "EINVAL should leave the destination buffer unchanged"
        );
    }
}
