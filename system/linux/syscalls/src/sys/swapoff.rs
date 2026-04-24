use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Disable swapping on an active swap area.
///
/// This wrapper spans the original Linux 1.0 x86 syscall slot `115` ABI and
/// the current native `swapoff(2)` entrypoints exported by this crate on x86
/// and aarch64. Current kernels keep the same single-pathname calling
/// convention.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.97.3
/// - Behavior changes: Linux 1.0 resolves `specialfile` with `namei()` and
///   matches either the active swap inode or the active swap block device;
///   current kernels open the pathname for read-write access, match the
///   resulting file mapping against active swap areas, and perform additional
///   accounting and teardown synchronization checks before deactivation.
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
/// - Current kernels require `CAP_SYS_ADMIN`.
///
/// # Behavior
/// - Resolves `specialfile` as the pathname of an active swap area.
/// - Returns `0` on success.
/// - Linux 1.0 deactivates the matching swap area after `try_to_unuse()`
///   succeeds and then releases its bookkeeping structures.
/// - Current kernels keep the one-argument ABI but add modern swap teardown
///   steps such as memory-accounting checks and RCU synchronization.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to disable swap.
/// - `EFAULT`: Linux 1.0 cannot read `specialfile` as a user pathname.
/// - `ENAMETOOLONG`: Linux 1.0 copies more than one page of pathname data.
/// - `ENOENT`: Linux 1.0 receives an empty pathname, or pathname lookup cannot
///   find a component.
/// - `ENOMEM`: Linux 1.0 cannot allocate pathname memory, or `try_to_unuse()`
///   cannot allocate a temporary page.
/// - `ENOTDIR`: Linux 1.0 finds a non-directory path component during lookup.
/// - `EACCES`: Linux 1.0 denies pathname traversal.
/// - `EINVAL`: the resolved pathname does not identify an active swap area.
///
/// Linux 1.0 forwards `namei()` errors directly, so filesystem-specific lookup
/// helpers may also return additional pathname-resolution errors. Current
/// kernels additionally propagate failures from opening `specialfile` with
/// `O_RDWR|O_LARGEFILE`, may return `ENOMEM` from
/// `security_vm_enough_memory_mm()`, and can return other errors from the
/// modern `try_to_unuse()` and teardown path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/swapon.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/swapfile.c?h=v7.0#n2769)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n130)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n579)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/swapfile.c?h=v6.18.18#n2868)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n130)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n579)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/swap.c?h=1.0#n674)
///
/// # Historical References
/// - First verified appearance: [Linux 0.97.3](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/swap.c?h=0.97.3#n531)
pub unsafe fn swapoff(specialfile: *const Char) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Swapoff, specialfile.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use crate::arch::current::{Sysno, syscall1};

    use super::swapoff;

    fn missing_path_bytes() -> Vec<u8> {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_swapoff_missing_{now}"));

        let mut bytes = path.as_os_str().as_encoded_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    #[test]
    fn test_swapoff_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 115;
        #[cfg(target_arch = "aarch64")]
        let expected = 225;

        assert_eq!(Sysno::Swapoff as isize, expected);
    }

    #[test]
    fn test_swapoff_matches_raw_syscall() {
        let path = missing_path_bytes();
        let ptr = path.as_ptr().cast::<Char>();

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let wrapped = unsafe { swapoff(ptr) };
        // SAFETY: this uses the same raw pointer as the wrapper under test.
        let raw =
            unsafe { syscall1(Sysno::Swapoff, ptr.addr() as isize) as i32 };

        assert_eq!(wrapped, raw, "swapoff wrapper should match raw syscall");
        assert!(wrapped < 0, "swapoff unexpectedly succeeded: {wrapped}");
    }
}
