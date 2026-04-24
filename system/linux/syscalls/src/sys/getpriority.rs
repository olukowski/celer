use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall2};

/// Return the highest matching scheduler priority for a process, process
/// group, or user selection.
///
/// This wrapper exposes the raw syscall result. Linux 1.0 returned the matched
/// task's internal `task_struct.priority` value, while current kernels
/// preserve the historical non-negative ABI by returning the encoded range
/// `40..1` instead of a negative-to-positive nice value.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 returned the highest matching internal task
///   priority; current kernels return the positive compatibility encoding
///   described in `kernel/sys.c`
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `which` selects the lookup class: `PRIO_PROCESS`, `PRIO_PGRP`, or
///   `PRIO_USER`.
/// - `who == 0` means the calling process, the caller's process group, or the
///   caller's real user ID, depending on `which`.
/// - On Linux 1.0, success returns the highest matching internal scheduler
///   priority across the selected tasks.
/// - On current kernels, success returns the highest matching encoded
///   compatibility priority.
///
/// # Errors
/// - `EINVAL`: `which` is not `PRIO_PROCESS`, `PRIO_PGRP`, or `PRIO_USER`.
/// - `ESRCH`: no task matches the selected `which` / `who` pair.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getpriority.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n329)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n111)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n390)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n329)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n111)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n390)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n82)
///
/// # Historical References
/// - Linux 1.0 priority selectors and limits:
///   [include/linux/resource.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/resource.h?h=1.0#n64)
pub fn getpriority(which: Int, who: Int) -> Int {
    // SAFETY: `getpriority` takes only integer arguments and has no
    // caller-visible memory-safety preconditions.
    unsafe { syscall2(Sysno::Getpriority, which as isize, who as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PRIO_PGRP, PRIO_PROCESS, PRIO_USER};

    use super::getpriority;
    use crate::arch::current::{Sysno, syscall2};

    #[test]
    fn test_getpriority_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 96;
        #[cfg(target_arch = "aarch64")]
        let expected = 141;
        #[cfg(target_arch = "x86_64")]
        let expected = 140;

        assert_eq!(Sysno::Getpriority as isize, expected);
    }

    #[test]
    fn test_getpriority_current_process_succeeds() {
        let priority = getpriority(PRIO_PROCESS, 0);

        assert!(
            priority > 0,
            "getpriority(PRIO_PROCESS, 0) should succeed, got {priority}"
        );
    }

    #[test]
    fn test_getpriority_current_process_group_succeeds() {
        let priority = getpriority(PRIO_PGRP, 0);

        assert!(
            priority > 0,
            "getpriority(PRIO_PGRP, 0) should succeed, got {priority}"
        );
    }

    #[test]
    fn test_getpriority_current_user_succeeds() {
        let priority = getpriority(PRIO_USER, 0);

        assert!(
            priority > 0,
            "getpriority(PRIO_USER, 0) should succeed, got {priority}"
        );
    }

    #[test]
    fn test_getpriority_rejects_invalid_selector() {
        let priority = getpriority(3, 0);

        assert_eq!(
            priority, -22,
            "expected EINVAL for invalid priority selector, got {priority}"
        );
    }

    #[test]
    fn test_getpriority_nonexistent_process_returns_esrch() {
        let priority = getpriority(PRIO_PROCESS, Int::MAX);

        assert_eq!(
            priority, -3,
            "expected ESRCH for nonexistent process, got {priority}"
        );
    }

    #[test]
    fn test_getpriority_invalid_selector_matches_raw_syscall() {
        let wrapped = getpriority(3, 0);
        // SAFETY: same integer-only arguments as the wrapper.
        let raw = unsafe { syscall2(Sysno::Getpriority, 3, 0) as Int };

        assert_eq!(wrapped, -22, "wrapped getpriority should return EINVAL");
        assert_eq!(raw, -22, "raw getpriority should return EINVAL");
    }
}
