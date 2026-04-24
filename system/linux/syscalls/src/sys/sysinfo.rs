#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;
use celer_system_linux_ctypes::{Int, Sysinfo};

use crate::arch::current::{Sysno, syscall1};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall1 as linux_1_0_syscall1,
};

/// Copy system load, memory, swap, and task summary information into the
/// caller-provided buffer.
///
/// # Safety
/// - `info`, when non-null, must be valid to write one `Sysinfo` value for
///   the duration of the syscall.
/// - `info`, when non-null, must not alias Rust references or other live Rust
///   allocations that the kernel may mutate in ways Rust cannot observe.
///
/// # Kernel Support
/// - Introduced: Linux 0.99.8
/// - Behavior changes: Linux 1.0 zeroed and returned the original 64-byte
///   `struct sysinfo` layout; current kernels still use syscall number `116`
///   on i386, but write the later 64-byte ABI tail with `totalhigh`,
///   `freehigh`, and `mem_unit` after `procs`.
/// - Availability: present on supported i386 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, writes one 64-byte `Sysinfo` record to `info`.
/// - `uptime` reports seconds since boot.
/// - `loads` reports the 1-, 5-, and 15-minute load averages as fixed-point
///   values shifted left by 16 bits.
/// - Current kernels report RAM and swap values in units of `mem_unit` bytes.
/// - The prefix through `procs` is compatible with Linux 1.0, but the current
///   i386 ABI uses the tail after `procs` for `pad`, `totalhigh`, `freehigh`,
///   `mem_unit`, and padding.
///
/// # Errors
/// - `EFAULT`: `info` is null or does not point to writable memory for one
///   `Sysinfo` record.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sysinfo.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n2959)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n2959)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/info.c?h=1.0#n17)
/// - First appearance: [Linux 0.99.8](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/info.c?h=0.99.8#n17)
/// - Linux 1.0 ABI layout:
///   [include/linux/kernel.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/kernel.h?h=1.0#n65)
/// - Current ABI layout:
///   [include/uapi/linux/sysinfo.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/sysinfo.h?h=v6.19#n8)
pub unsafe fn sysinfo(info: *mut Sysinfo) -> Int {
    // SAFETY: `info` is forwarded to the kernel exactly as provided by the
    // caller, which must uphold the pointer validity and aliasing
    // requirements documented above.
    unsafe { syscall1(Sysno::Sysinfo, info.addr() as isize) as Int }
}

/// Copy Linux 1.0 system load, memory, swap, and task-table summary
/// information into the caller-provided buffer.
///
/// This wrapper uses syscall slot `116` with the Linux 1.0
/// [`Linux10Sysinfo`] layout. Current kernels use the same slot with the
/// current i386 [`Sysinfo`] layout, exposed by [`sysinfo`].
///
/// # Safety
/// - `info`, when non-null, must be valid to write one [`Linux10Sysinfo`] value
///   for the duration of the syscall.
/// - `info`, when non-null, must not alias live Rust references or other memory
///   that would violate Rust's aliasing rules while the kernel may write
///   through that pointer.
///
/// # Kernel Support
/// - Introduced: Linux 0.99.8
/// - Behavior changes: current kernels still use syscall number `116` on i386,
///   but write the later 64-byte ABI tail with `totalhigh`, `freehigh`, and
///   `mem_unit` after `procs`.
/// - Availability: correct only for Linux 1.0 x86 kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On Linux 1.0, success writes one 64-byte `struct sysinfo` record to
///   `info`.
/// - `uptime` reports seconds since boot.
/// - `loads` reports the 1-, 5-, and 15-minute load averages as fixed-point
///   values shifted left by 16 bits.
/// - RAM and swap fields report byte counts.
/// - `procs` counts occupied task slots in the kernel task table.
///
/// # Errors
/// - `EFAULT`: `info` is null or does not point to writable memory for one
///   Linux 1.0 `struct sysinfo` record.
///
/// # References
/// - Linux 1.0 syscall number table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n125)
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/info.c?h=1.0#n17)
/// - First appearance:
///   [Linux 0.99.8](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/info.c?h=0.99.8#n17)
/// - Linux 1.0 ABI layout:
///   [include/linux/kernel.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/kernel.h?h=1.0#n65)
#[cfg(target_arch = "x86")]
pub unsafe fn sysinfo_1_0(info: *mut Linux10Sysinfo) -> Int {
    // SAFETY: `info` is forwarded to the kernel exactly as provided by the
    // caller, which must uphold the pointer validity and aliasing
    // requirements documented above.
    unsafe {
        linux_1_0_syscall1(Linux10Sysno::Sysinfo, info.addr() as isize) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Sysinfo;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;

    use crate::arch::current::Sysno;
    #[cfg(target_arch = "x86")]
    use crate::arch::linux_1_0::Sysno as Linux10Sysno;

    use super::sysinfo;
    #[cfg(target_arch = "x86")]
    use super::sysinfo_1_0;

    #[test]
    fn test_sysinfo_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Sysinfo as isize, 116);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Sysinfo as isize, 179);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_sysinfo_sysno() {
        assert_eq!(Linux10Sysno::Sysinfo as isize, 116);
    }

    #[test]
    fn test_sysinfo_layout() {
        #[cfg(target_arch = "x86")]
        let expected = (64, 4, 16, 40);
        #[cfg(target_arch = "aarch64")]
        let expected = (112, 8, 32, 80);

        assert_eq!(core::mem::size_of::<Sysinfo>(), expected.0);
        assert_eq!(core::mem::offset_of!(Sysinfo, uptime), 0);
        assert_eq!(core::mem::offset_of!(Sysinfo, loads), expected.1);
        assert_eq!(core::mem::offset_of!(Sysinfo, totalram), expected.2);
        assert_eq!(core::mem::offset_of!(Sysinfo, procs), expected.3);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_sysinfo_layout() {
        assert_eq!(core::mem::size_of::<Linux10Sysinfo>(), 64);
        assert_eq!(core::mem::offset_of!(Linux10Sysinfo, uptime), 0);
        assert_eq!(core::mem::offset_of!(Linux10Sysinfo, loads), 4);
        assert_eq!(core::mem::offset_of!(Linux10Sysinfo, totalram), 16);
        assert_eq!(core::mem::offset_of!(Linux10Sysinfo, procs), 40);
        assert_eq!(core::mem::offset_of!(Linux10Sysinfo, _f), 42);
    }

    #[test]
    fn test_sysinfo() {
        let mut info = Sysinfo {
            uptime: -1,
            loads: [0; 3],
            totalram: 0,
            freeram: 0,
            sharedram: 0,
            bufferram: 0,
            totalswap: 0,
            freeswap: 0,
            procs: 0,
            pad: 0,
            totalhigh: 0,
            freehigh: 0,
            mem_unit: 0,
            #[cfg(target_arch = "x86")]
            _f: [1; 8],
            #[cfg(target_arch = "aarch64")]
            _f: [],
        };

        // SAFETY: `info` is a valid writable output buffer for one `Sysinfo`
        // value and does not alias any Rust reference the kernel may mutate.
        let ret = unsafe { sysinfo(&raw mut info) };

        assert_eq!(ret, 0, "sysinfo failed: {ret}");
        assert!(info.uptime >= 0, "uptime should be non-negative");
        assert_eq!(info.loads.len(), 3);
        assert!(info.procs > 0, "procs should report at least one task");
        assert!(info.totalram >= info.freeram);
        assert!(info.totalswap >= info.freeswap);
        assert!(info.mem_unit > 0, "mem_unit should be non-zero");
    }

    #[test]
    fn test_sysinfo_null_pointer() {
        // SAFETY: a null pointer is permitted and the kernel reports invalid
        // output pointers as `EFAULT`.
        let ret = unsafe { sysinfo(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_sysinfo_null_pointer() {
        // SAFETY: a null pointer is permitted and the kernel reports invalid
        // output pointers as `EFAULT`.
        let ret = unsafe { sysinfo_1_0(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
