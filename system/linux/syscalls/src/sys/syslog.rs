use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall3};

/// Read from or control the kernel log buffer through the historical
/// `sys_syslog` multiplexed syscall ABI.
///
/// This wrapper spans the original Linux 1.0 x86 syscall slot `103` ABI and
/// the current native `syslog(2)` entrypoints exported by this crate on x86
/// and aarch64. Linux 1.0 accepts command types `0..=8`; current kernels keep
/// those commands and add `9` (`SIZE_UNREAD`) and `10` (`SIZE_BUFFER`).
///
/// # Safety
/// - If `type_` is `2`, `3`, or `4` and `len > 0`, `buf` must point to
///   writable memory for `len` bytes for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.96a
/// - Behavior changes: Linux 1.0 exposes only command types `0..=8`;
///   current kernels keep those historical commands and add
///   `SIZE_UNREAD` (`9`) and `SIZE_BUFFER` (`10`).
/// - Availability: present on supported x86 and aarch64 Linux kernels
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
/// - `EINTR`: command type `2` was waiting for log data and a signal handler
///   interrupted the wait without restarting the syscall. On Linux 1.0 this
///   path uses internal `ERESTARTSYS`; the signal-return path either exposes
///   `EINTR` to user space or restarts the syscall.
/// - `ENOMEM`: on current kernels, command types `2`, `3`, and `4` can fail
///   while allocating temporary log-copy buffers.
/// - Current kernels can also return additional policy-dependent errors from
///   `security_syslog()`.
///
/// # References
/// - Stable x86 table:
///   [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n118)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n327)
/// - Stable entry:
///   [v7.0 syslog](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n1853)
/// - Stable helper:
///   [v7.0 do_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n1740)
/// - Stable permission check:
///   [v7.0 check_syslog_permissions](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/printk/printk.c?h=v7.0#n617)
/// - LTS x86 table:
///   [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n118)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n327)
/// - LTS entry:
///   [v6.18.18 syslog](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/printk/printk.c?h=v6.18.18#n1851)
/// - LTS helper:
///   [v6.18.18 do_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/printk/printk.c?h=v6.18.18#n1738)
/// - First stable:
///   [Linux 1.0 sys_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/printk.c?h=1.0#n55)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.96a sys_syslog](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/printk.c?h=0.96a#n27)
/// - Linux 1.0 signal restart handling:
///   [kernel/signal.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n384)
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
        #[cfg(target_arch = "x86")]
        let expected = 103;
        #[cfg(target_arch = "aarch64")]
        let expected = 116;
        #[cfg(target_arch = "x86_64")]
        let expected = 103;

        assert_eq!(Sysno::Syslog as isize, expected);
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
