use celer_system_linux_ctypes::{
    Long, Rlimit, UnsignedInt, linux_1_0::Rlimit as Linux10Rlimit,
};

use crate::arch::{
    current::{Sysno, syscall2},
    linux_1_0::{Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2},
};

/// Set the calling task's soft and hard resource limits for one resource.
///
/// This wrapper exposes the current x86 `setrlimit(2)` ABI at syscall slot
/// `75`. Linux 1.0 used the same syscall slot with a signed 32-bit
/// `struct rlimit` layout, exposed separately as
/// [`crate::sys::linux_1_0::setrlimit`].
///
/// # Safety
/// - `rlim` must be valid to read one [`Rlimit`] for the duration of the
///   syscall.
///
/// # Kernel Support
/// - Historical slot introduced: Linux 1.0
/// - Behavior changes: current kernels use unsigned `rlim_t` fields, validate
///   `rlim_cur <= rlim_max`, reject malformed user pointers before limit
///   checks, cap `RLIMIT_NOFILE` against `sysctl_nr_open`, and may deny
///   updates through an LSM hook.
/// - Availability: this wrapper is ABI-correct for current supported x86 Linux
///   kernels; it is not ABI-compatible with Linux 1.0.
///
/// # Required Privileges
/// - Current kernels require privilege to raise the hard limit and also reject
///   oversize `RLIMIT_NOFILE` requests before the capability check.
///
/// # Behavior
/// - `resource` selects one entry in the calling task's resource-limit table.
/// - On success, the kernel replaces both the soft and hard limits for that
///   resource with the unsigned `rlim_t` values from `rlim`.
/// - Current kernels accept resource IDs in the current `RLIM_NLIMITS` range.
/// - `UnsignedLong::MAX` is the current i386 bit pattern for
///   `RLIM_INFINITY`.
///
/// # Errors
/// - `EFAULT`: on current kernels, `rlim` is not readable for one
///   [`Rlimit`].
/// - `EINVAL`: `resource` is out of range, or on current kernels
///   `rlim.rlim_cur > rlim.rlim_max`.
/// - `EPERM`: the requested update exceeds the caller's authority, including
///   attempts to raise the hard limit without sufficient privilege or to set
///   `RLIMIT_NOFILE` above the kernel-wide maximum.
///
/// Current kernels may also return an LSM-defined negative errno from
/// `security_task_setrlimit()`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getrlimit.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1794)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1794)
/// - Current i386 `struct rlimit`: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/resource.h?h=v6.19#n43)
///
pub unsafe fn setrlimit(resource: UnsignedInt, rlim: *const Rlimit) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Setrlimit, resource as isize, rlim.addr() as isize)
            as Long
    }
}

/// Set a Linux 1.0 task's soft and hard resource limits for one resource.
///
/// This is the historical Linux 1.0 ABI at syscall slot `75`, which used
/// signed `int` fields in `struct rlimit`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Availability: correct only for Linux 1.0 x86 kernels; current x86 Linux
///   uses the same syscall number for the unsigned current `setrlimit(2)` ABI
///   exposed by [`setrlimit`].
///
/// # Safety
/// - `rlim` must be valid to read one [`Linux10Rlimit`] for the duration of
///   the syscall.
///
/// # Required Privileges
/// - Linux 1.0 requires superuser privilege when either requested limit
///   exceeds the current hard limit.
///
/// # Behavior
/// - `resource` selects one of the six Linux 1.0 resource-limit slots:
///   `RLIMIT_CPU` through `RLIMIT_RSS`.
/// - On success, the kernel replaces both the soft and hard limits for that
///   resource with the signed `int` values from `rlim`.
/// - Linux 1.0 does not reject `rlim_cur > rlim_max` in the syscall body.
/// - Linux 1.0 reads `rlim` with `get_fs_long()` and does not contain an
///   explicit `EFAULT` return path in this syscall body.
///
/// # Errors
/// - `EINVAL`: `resource` is outside Linux 1.0's `RLIM_NLIMITS` table.
/// - `EPERM`: the requested update exceeds the caller's authority.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n701)
/// - Linux 1.0 `struct rlimit`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/resource.h?h=1.0#n60)
pub unsafe fn setrlimit_1_0(
    resource: UnsignedInt,
    rlim: *const Linux10Rlimit,
) -> Long {
    // SAFETY: the wrapper forwards the raw historical ABI argument.
    unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Setrlimit,
            resource as isize,
            rlim.addr() as isize,
        ) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::fs;

    use celer_system_linux_ctypes::{
        Int, Rlimit, UnsignedInt, UnsignedLong,
        linux_1_0::Rlimit as Linux10Rlimit,
    };

    use crate::arch::linux_1_0::Sysno as Linux10Sysno;

    use crate::sys::test_support::process_global_state_guard;

    use super::{setrlimit, setrlimit_1_0};

    const RLIMIT_CPU: UnsignedInt = 0;
    const RLIMIT_NOFILE: UnsignedInt = 7;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    unsafe extern "C" {
        fn getrlimit(resource: UnsignedInt, rlim: *mut Rlimit) -> Int;
    }

    #[test]
    fn test_setrlimit_layout() {
        assert_eq!(core::mem::size_of::<Rlimit>(), 8);
        assert_eq!(core::mem::align_of::<Rlimit>(), 4);
        assert_eq!(core::mem::offset_of!(Rlimit, rlim_cur), 0);
        assert_eq!(core::mem::offset_of!(Rlimit, rlim_max), 4);
    }

    #[test]
    fn test_linux_1_0_setrlimit_layout() {
        assert_eq!(Linux10Sysno::Setrlimit as isize, 75);
        assert_eq!(core::mem::size_of::<Linux10Rlimit>(), 8);
        assert_eq!(core::mem::align_of::<Linux10Rlimit>(), 4);
        assert_eq!(core::mem::offset_of!(Linux10Rlimit, rlim_cur), 0);
        assert_eq!(core::mem::offset_of!(Linux10Rlimit, rlim_max), 4);
    }

    #[test]
    fn test_setrlimit_invalid_resource() {
        let limits = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe { setrlimit(CURRENT_RLIM_NLIMITS, &raw const limits) };

        assert_eq!(
            ret, -22,
            "setrlimit should reject an out-of-range resource"
        );
    }

    #[test]
    fn test_setrlimit_rejects_soft_limit_above_hard_limit() {
        let limits = Rlimit {
            rlim_cur: 1,
            rlim_max: 0,
        };

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe { setrlimit(RLIMIT_CPU, &raw const limits) };

        assert_eq!(ret, -22, "setrlimit should reject rlim_cur > rlim_max");
    }

    #[test]
    fn test_setrlimit_accepts_existing_limit_pair() {
        let _guard = process_global_state_guard();
        let mut limits = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: `limits` is writable for one `struct rlimit`.
        let get_ret = unsafe { getrlimit(RLIMIT_CPU, &raw mut limits) };
        assert_eq!(get_ret, 0, "getrlimit(RLIMIT_CPU) failed: {get_ret}");

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe { setrlimit(RLIMIT_CPU, &raw const limits) };

        assert_eq!(ret, 0, "setrlimit should accept the current limit pair");
    }

    #[test]
    fn test_setrlimit_rejects_rlimit_nofile_above_nr_open() {
        let _guard = process_global_state_guard();
        let nr_open = fs::read_to_string("/proc/sys/fs/nr_open")
            .expect("failed to read /proc/sys/fs/nr_open")
            .trim()
            .parse::<u64>()
            .expect("nr_open should parse as an integer");
        let requested: UnsignedLong = nr_open
            .checked_add(1)
            .expect("nr_open + 1 should fit in u64")
            .try_into()
            .expect("nr_open + 1 should fit in UnsignedLong");
        let limits = Rlimit {
            rlim_cur: requested,
            rlim_max: requested,
        };

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe { setrlimit(RLIMIT_NOFILE, &raw const limits) };

        assert_eq!(
            ret, -1,
            "setrlimit should reject RLIMIT_NOFILE above /proc/sys/fs/nr_open"
        );
    }

    #[test]
    fn test_linux_1_0_setrlimit_matches_current_raw_slot() {
        let limits = Linux10Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret =
            unsafe { setrlimit_1_0(CURRENT_RLIM_NLIMITS, &raw const limits) };

        assert_eq!(
            ret, -22,
            "Linux 1.0 setrlimit wrapper should still call syscall slot 75"
        );
    }
}
