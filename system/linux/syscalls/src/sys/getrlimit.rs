use celer_system_linux_ctypes::{Int, Rlimit, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Copy the current process resource limits for one Linux resource.
///
/// On x86, this wrapper uses syscall number `76`. Linux 1.0 used a signed
/// 32-bit `struct rlimit` layout for that slot; current x86 kernels implement
/// the same slot as the legacy `old_getrlimit` entrypoint, whose unsigned
/// 32-bit output fields are clamped to the historical signed range. On
/// aarch64, this wrapper uses syscall number `163`, the native
/// `sys_getrlimit` entrypoint with the native `struct rlimit` layout.
///
/// # Safety
/// - `rlim` must point to writable memory for one [`Rlimit`] value for the
///   duration of the syscall.
/// - `rlim` must not alias live Rust references or other memory that would
///   violate Rust's aliasing rules while the kernel may write through that
///   pointer.
///
/// # Kernel Support
/// - Introduced: Linux 1.0 on i386; present from the initial aarch64 syscall
///   table
/// - Behavior changes: newer x86 kernels keep syscall number 76 as the legacy
///   `old_getrlimit` entrypoint, accept the current `RLIM_NLIMITS` resource
///   range, and clamp returned values to `0x7fffffff` when the modern
///   internal limit exceeds the historical signed 32-bit ABI range.
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On Linux 1.0, `resource` selected one of the six historical resource
///   slots: `RLIMIT_CPU` through `RLIMIT_RSS`.
/// - On current kernels, this entrypoint accepts the current `RLIM_NLIMITS`
///   range instead of the six Linux 1.0 slots.
/// - On success, the kernel writes the current soft limit to `rlim_cur` and
///   the hard limit to `rlim_max`.
/// - Linux 1.0 validates `resource` before touching `rlim`.
/// - On i386, this wrapper targets the historical syscall entrypoint, not the
///   later `ugetrlimit` or `prlimit64` interfaces.
///
/// # Errors
/// - `EINVAL`: `resource` is out of range for the kernel's `RLIM_NLIMITS`
///   table.
/// - `EFAULT`: `rlim` is not writable for one [`Rlimit`] value.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getrlimit.2.html)
/// - Stable native implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n1563)
/// - Stable x86 compatibility implementation: [v7.0 old_getrlimit](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n1628)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n91)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n437)
/// - LTS native implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1563)
/// - LTS x86 compatibility implementation: [v6.18.18 old_getrlimit](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1628)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n91)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n437)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n685)
///
/// # Historical References
/// - Linux 1.0 resource constants and `struct rlimit`: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/resource.h?h=1.0#n42)
pub unsafe fn getrlimit(resource: UnsignedInt, rlim: *mut Rlimit) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Getrlimit, resource as isize, rlim.addr() as isize)
            as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Rlimit, UnsignedInt};

    use crate::arch::current::Sysno;

    use super::getrlimit;

    const RLIMIT_CPU: UnsignedInt = 0;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    #[test]
    fn test_getrlimit_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Getrlimit as isize, 76);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Getrlimit as isize, 163);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(Sysno::Getrlimit as isize, 97);
    }

    #[test]
    fn test_getrlimit_layout() {
        assert_eq!(core::mem::offset_of!(Rlimit, rlim_cur), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(core::mem::size_of::<Rlimit>(), 8);
            assert_eq!(core::mem::align_of::<Rlimit>(), 4);
            assert_eq!(core::mem::offset_of!(Rlimit, rlim_max), 4);
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            assert_eq!(core::mem::size_of::<Rlimit>(), 16);
            assert_eq!(core::mem::align_of::<Rlimit>(), 8);
            assert_eq!(core::mem::offset_of!(Rlimit, rlim_max), 8);
        }
    }

    #[test]
    fn test_getrlimit_cpu_success() {
        let mut rlim = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: `rlim` is writable for a full `Rlimit`.
        let ret = unsafe { getrlimit(RLIMIT_CPU, &raw mut rlim) };

        assert_eq!(ret, 0, "getrlimit failed for RLIMIT_CPU: {ret}");
        assert!(
            rlim.rlim_cur <= rlim.rlim_max,
            "soft limit should not exceed hard limit: {:?}",
            rlim
        );
    }

    #[test]
    fn test_getrlimit_invalid_resource_returns_einval_without_writing() {
        let mut rlim = Rlimit {
            rlim_cur: 123,
            rlim_max: 456,
        };

        // SAFETY: `rlim` is writable for a full `Rlimit`.
        let ret = unsafe { getrlimit(CURRENT_RLIM_NLIMITS, &raw mut rlim) };

        assert_eq!(
            ret, -22,
            "expected EINVAL from invalid resource, got {ret}"
        );
        assert_eq!(
            rlim,
            Rlimit {
                rlim_cur: 123,
                rlim_max: 456,
            }
        );
    }

    #[test]
    fn test_getrlimit_null_pointer_returns_efault() {
        // SAFETY: this test intentionally passes a null pointer to verify the
        // kernel's `EFAULT` path.
        let ret = unsafe { getrlimit(RLIMIT_CPU, core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
