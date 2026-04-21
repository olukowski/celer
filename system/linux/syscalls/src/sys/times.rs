use celer_system_linux_ctypes::{Long, Tms};

use crate::arch::current::{Sysno, syscall1};

/// Return process and child CPU time statistics.
///
/// # Safety
/// - `tbuf`, when non-null, must be valid to write a single [`Tms`] value for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - If `tbuf` is non-null, the kernel stores the current process and child
///   CPU times in the pointed-to [`Tms`] structure.
/// - The syscall returns a clock tick count derived from the current jiffies
///   value.
/// - The raw return value may be negative on success after wraparound, so
///   callers must not treat every negative value from this syscall as an
///   errno.
/// - Passing a null pointer skips the copy-out and still returns the clock
///   tick count.
///
/// # Errors
/// - `EFAULT`: `tbuf` is non-null but not writable for a full [`Tms`] value.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/times.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1064)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1064)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n58)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n156)
pub unsafe fn times(tbuf: *mut Tms) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Times, tbuf.addr() as isize) as Long }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Tms;

    use super::times;

    #[test]
    fn test_times_null() {
        // SAFETY: passing a null pointer is explicitly supported by `times`.
        let _ = unsafe { times(core::ptr::null_mut()) };
    }

    #[test]
    fn test_times_write_through_pointer() {
        let mut tms = Tms {
            tms_utime: 0,
            tms_stime: 0,
            tms_cutime: 0,
            tms_cstime: 0,
        };

        // SAFETY: `tms` is writable for a full `Tms`.
        let now = unsafe { times(&raw mut tms) };

        assert_ne!(now, -14, "times should not fail with EFAULT: {now}");
    }
}
