use celer_system_linux_ctypes::{Long, Rlimit, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Set the calling task's soft and hard resource limits for one resource.
///
/// This is the historical `setrlimit` syscall entry point from Linux 1.0.
///
/// # Kernel Support
/// - First stable: Linux 1.0
/// - Behavior changes: current kernels validate `rlim_cur <= rlim_max`,
///   reject malformed user pointers before limit checks, cap `RLIMIT_NOFILE`
///   against `sysctl_nr_open`, and may deny updates through an LSM hook.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 required superuser privilege when either requested limit
///   exceeded the current hard limit.
/// - Current kernels still require privilege to raise the hard limit and also
///   reject oversize `RLIMIT_NOFILE` requests before the capability check.
///
/// # Behavior
/// - `resource` selects one entry in the calling task's resource-limit table.
/// - On success, the kernel replaces both the soft and hard limits for that
///   resource with the values from `rlim`.
/// - Linux 1.0 accepted only resource IDs `0..=5`; current kernels accept
///   `0..=15`.
/// - Linux 1.0 did not reject `rlim_cur > rlim_max` in the syscall body.
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
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n701)
///
pub fn setrlimit(resource: UnsignedInt, rlim: *const Rlimit) -> Long {
    // SAFETY: the wrapper forwards the raw user pointer exactly as the kernel
    // ABI expects; invalid pointers are reported by the kernel as syscall
    // errors rather than causing Rust UB.
    unsafe {
        syscall2(Sysno::Setrlimit, resource as isize, rlim.addr() as isize)
            as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use celer_system_linux_ctypes::{Int, Rlimit, UnsignedInt};

    use super::setrlimit;

    const RLIMIT_CPU: UnsignedInt = 0;
    const RLIMIT_NOFILE: UnsignedInt = 7;
    const CURRENT_RLIM_NLIMITS: UnsignedInt = 16;

    static SETRLIMIT_LOCK: Mutex<()> = Mutex::new(());

    unsafe extern "C" {
        fn getrlimit(resource: UnsignedInt, rlim: *mut Rlimit) -> Int;
    }

    #[test]
    fn test_setrlimit_invalid_resource() {
        let limits = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        let ret = setrlimit(CURRENT_RLIM_NLIMITS, &raw const limits);

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

        let ret = setrlimit(RLIMIT_CPU, &raw const limits);

        assert_eq!(ret, -22, "setrlimit should reject rlim_cur > rlim_max");
    }

    #[test]
    fn test_setrlimit_null_pointer_faults_on_current_kernels() {
        let ret = setrlimit(RLIMIT_CPU, core::ptr::null());

        assert_eq!(ret, -14, "setrlimit(null) should fail with EFAULT");
    }

    #[test]
    fn test_setrlimit_accepts_existing_limit_pair() {
        let _guard = SETRLIMIT_LOCK.lock().unwrap();
        let mut limits = Rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: `limits` is writable for one `struct rlimit`.
        let get_ret = unsafe { getrlimit(RLIMIT_CPU, &raw mut limits) };
        assert_eq!(get_ret, 0, "getrlimit(RLIMIT_CPU) failed: {get_ret}");

        let ret = setrlimit(RLIMIT_CPU, &raw const limits);

        assert_eq!(ret, 0, "setrlimit should accept the current limit pair");
    }

    #[test]
    fn test_setrlimit_rejects_rlimit_nofile_above_nr_open() {
        let nr_open = fs::read_to_string("/proc/sys/fs/nr_open")
            .expect("failed to read /proc/sys/fs/nr_open")
            .trim()
            .parse::<u64>()
            .expect("nr_open should parse as an integer");
        let requested = nr_open
            .checked_add(1)
            .expect("nr_open + 1 should fit in u64");
        let limits = Rlimit {
            rlim_cur: requested as _,
            rlim_max: requested as _,
        };

        let ret = setrlimit(RLIMIT_NOFILE, &raw const limits);

        assert_eq!(
            ret, -1,
            "setrlimit should reject RLIMIT_NOFILE above /proc/sys/fs/nr_open"
        );
    }
}
