use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall3};

/// Read from or control the kernel log buffer through the historical
/// `sys_syslog` multiplexed syscall ABI.
///
/// This wrapper targets the original Linux 1.0 syscall slot `103` ABI.
/// Linux 1.0 accepts command types `0..=8`; current x86 kernels keep the same
/// syscall number and add command types `9` (`SIZE_UNREAD`) and
/// `10` (`SIZE_BUFFER`).
///
/// # Safety
/// - If `type_` is `2`, `3`, or `4` and `len > 0`, `buf` must point to
///   writable memory for `len` bytes for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 exposes only command types `0..=8`;
///   current x86 kernels keep those historical commands and add
///   `SIZE_UNREAD` (`9`) and `SIZE_BUFFER` (`10`)
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 allows unprivileged callers only for command type `3`
///   (`READ_ALL`); all other command types require a superuser caller.
/// - Current kernels generally require `CAP_SYSLOG` for restricted actions
///   and may additionally deny access through `security_syslog()`.
///
/// # Behavior
/// - `type_` selects the sub-operation performed by the syscall.
/// - Linux 1.0 recognizes only command types `0..=8`; current kernels also
///   recognize `9` and `10`.
/// - For read-like command types `2`, `3`, and `4`, `len` is the maximum
///   byte count copied into `buf`.
/// - For command type `8`, Linux 1.0 treats `len` as the requested console
///   loglevel and accepts `0..=8`; current kernels require `1..=8`.
/// - Permission checks for restricted actions happen before command
///   validation and before the zero-length fast path, so `len == 0` does not
///   bypass `EPERM` and unsupported restricted `type_` values may still
///   return `EPERM`.
/// - On success, command types `0`, `1`, `5`, `6`, `7`, and `8` return `0`.
/// - On success, read-like command types return the number of bytes copied.
/// - On current kernels, command type `9` returns the unread byte count and
///   command type `10` returns the kernel log buffer size.
/// - Linux 1.0 reads raw bytes from a fixed 4 KiB ring buffer. Current
///   kernels format log records and may return short reads.
/// - Command type `2` can block waiting for additional log data.
///
/// # Errors
/// - `EPERM`: the selected command requires privilege and the caller lacks
///   it.
/// - `EINVAL`: `type_` is unsupported, a read-like command uses a null `buf`
///   with `len > 0`, `len` is negative for a read-like command, or command
///   type `8` uses an unsupported `len` value.
/// - `EFAULT`: a read-like command uses a `buf` pointer that is not writable
///   for the requested byte count.
/// - `ERESTARTSYS`: on Linux 1.0, command type `2` was waiting for log data
///   and a signal interrupted the sleep before data became available.
/// - `ENOMEM`: on current kernels, command types `2`, `3`, and `4` can fail
///   while allocating temporary log-copy buffers.
/// - Current kernels can also return additional policy-dependent errors from
///   `security_syslog()`.
///
/// # References
/// - Stable entry:
///   [v7.0 syslog](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n1853)
/// - Stable helper:
///   [v7.0 do_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n1740)
/// - Stable permission check:
///   [v7.0 check_syslog_permissions](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n617)
/// - LTS entry:
///   [v6.18.18 syslog](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/printk/printk.c?h=v6.18.18#n1851)
/// - LTS helper:
///   [v6.18.18 do_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/printk/printk.c?h=v6.18.18#n1738)
/// - First stable:
///   [Linux 1.0 sys_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/printk.c?h=1.0#n55)
///
/// # Historical References
/// - Linux 1.0 syscall table:
///   [kernel/sched.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n140)
pub unsafe fn syslog(type_: Int, buf: *mut Char, len: Int) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::Syslog,
            type_ as isize,
            buf.addr() as isize,
            len as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, Int};

    use crate::arch::current::{Sysno, syscall3};

    use super::syslog;

    #[test]
    fn test_syslog_sysno() {
        assert_eq!(Sysno::Syslog as isize, 103);
    }

    #[test]
    fn test_syslog_invalid_type_matches_raw_syscall() {
        // SAFETY: the invalid command causes the kernel to reject the call
        // before interpreting `buf`, but privilege checks may still run first.
        let wrapped = unsafe { syslog(11, core::ptr::null_mut(), 0) };
        // SAFETY: uses the same invalid command and null pointer as the
        // wrapper under test.
        let raw = unsafe {
            syscall3(
                Sysno::Syslog,
                11,
                core::ptr::null_mut::<Char>().addr() as isize,
                0,
            )
        } as Int;

        assert_eq!(wrapped, raw, "wrapped syslog should match raw syscall");
    }
}
