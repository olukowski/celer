use celer_system_linux_ctypes::{Int, Rlimit, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Copy the current process resource limits for one historical Linux resource.
///
/// This wrapper exposes the original i386 syscall number 76 ABI from Linux
/// 1.0, which on current x86 kernels is implemented by the legacy
/// `old_getrlimit` entrypoint and uses the 32-bit `struct rlimit` layout in
/// [`Rlimit`].
///
/// # Safety
/// - `rlim` must point to writable memory for one [`Rlimit`] value for the
///   duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: newer x86 kernels keep syscall number 76 as the legacy
///   `old_getrlimit` entrypoint, accept the current `RLIM_NLIMITS` resource
///   range, and clamp returned values to `0x7fffffff` when the modern
///   internal limit exceeds the historical ABI range.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On Linux 1.0, `resource` selected one of the six historical resource
///   slots: `RLIMIT_CPU` through `RLIMIT_RSS`.
/// - On current kernels, this legacy entrypoint accepts the current
///   `RLIM_NLIMITS` range instead of the six Linux 1.0 slots.
/// - On success, the kernel writes the current soft limit to `rlim_cur` and
///   the hard limit to `rlim_max`.
/// - Linux 1.0 validates `resource` before touching `rlim`.
/// - This wrapper targets the historical syscall entrypoint, not the later
///   `ugetrlimit` or `prlimit64` interfaces.
///
/// # Errors
/// - `EINVAL`: `resource` is out of range for the kernel's `RLIM_NLIMITS`
///   table.
/// - `EFAULT`: `rlim` is not writable for one [`Rlimit`] value.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getrlimit.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1628)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1628)
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

    use super::getrlimit;

    const RLIMIT_CPU: UnsignedInt = 0;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    #[test]
    fn test_getrlimit_layout() {
        assert_eq!(core::mem::size_of::<Rlimit>(), 8);
        assert_eq!(core::mem::align_of::<Rlimit>(), 4);
        assert_eq!(core::mem::offset_of!(Rlimit, rlim_cur), 0);
        assert_eq!(core::mem::offset_of!(Rlimit, rlim_max), 4);
    }

    #[test]
    fn test_getrlimit_cpu_success() {
        let mut rlim = Rlimit {
            rlim_cur: u32::MAX,
            rlim_max: u32::MAX,
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
