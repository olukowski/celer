use celer_system_linux_ctypes::{Int, Sysinfo};

use crate::arch::current::{Sysno, syscall1};

/// Copy Linux 1.0 system load, memory, swap, and task-table summary
/// information into the caller-provided buffer.
///
/// # Safety
/// - `info`, when non-null, must be valid to write one `Sysinfo` value for
///   the duration of the syscall.
/// - `info`, when non-null, must not alias Rust references or other live Rust
///   allocations that the kernel may mutate in ways Rust cannot observe.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 zeroed and returned the original 64-byte
///   `struct sysinfo` layout; current kernels still use syscall number `116`
///   on i386, but write the later 64-byte ABI tail with `totalhigh`,
///   `freehigh`, and `mem_unit` after `procs`.
/// - Availability: present on supported i386 Linux kernels, but this wrapper's
///   type models the original Linux 1.0 layout only
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On Linux 1.0, success writes one 64-byte `Sysinfo` record to `info`.
/// - In the Linux 1.0 ABI, `uptime` reports seconds since boot.
/// - In the Linux 1.0 ABI, `loads` reports the 1-, 5-, and 15-minute load
///   averages as fixed-point values shifted left by 16 bits.
/// - In the Linux 1.0 ABI, the RAM and swap fields report byte counts.
/// - In the Linux 1.0 ABI, `procs` counts occupied task slots in the kernel
///   task table.
/// - On newer kernels, the prefix through `procs` stays compatible, but bytes
///   after `procs` are used for the newer ABI tail and memory values may be
///   scaled by a `mem_unit` field that this historical type does not expose.
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
/// - Linux 1.0 ABI layout:
///   [include/linux/kernel.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/kernel.h?h=1.0#n65)
pub unsafe fn sysinfo(info: *mut Sysinfo) -> Int {
    // SAFETY: `info` is forwarded to the kernel exactly as provided by the
    // caller, which must uphold the pointer validity and aliasing
    // requirements documented above.
    unsafe { syscall1(Sysno::Sysinfo, info.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Sysinfo;

    use crate::arch::current::Sysno;

    use super::sysinfo;

    #[test]
    fn test_sysinfo_sysno() {
        assert_eq!(Sysno::Sysinfo as isize, 116);
    }

    #[test]
    fn test_sysinfo_layout() {
        assert_eq!(core::mem::size_of::<Sysinfo>(), 64);
        assert_eq!(core::mem::offset_of!(Sysinfo, uptime), 0);
        assert_eq!(core::mem::offset_of!(Sysinfo, loads), 4);
        assert_eq!(core::mem::offset_of!(Sysinfo, totalram), 16);
        assert_eq!(core::mem::offset_of!(Sysinfo, procs), 40);
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
            _f: [1; 22],
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
    }

    #[test]
    fn test_sysinfo_null_pointer() {
        // SAFETY: a null pointer is permitted and the kernel reports invalid
        // output pointers as `EFAULT`.
        let ret = unsafe { sysinfo(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
