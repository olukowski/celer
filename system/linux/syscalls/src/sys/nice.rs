use celer_system_linux_ctypes::{Int, Long};

use crate::arch::current::{Sysno, syscall1};

/// Change the current process nice value by a relative increment.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for non-negative increments, but LSM policy may still deny the call.
/// - Decreasing the nice value requires sufficient privilege.
///
/// # Behavior
/// - The kernel adds `increment` to the current task's nice value and clamps
///   the result to the scheduler's allowed range.
/// - On success, the syscall returns `0`.
/// - This wrapper exposes the raw syscall result, not libc `nice()` semantics.
///
/// # Errors
/// - `EPERM`: lowering the nice value is not permitted for the caller, or an
///   LSM denies the update.
/// - Other LSM hook errors may be returned directly.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/nice.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sched/syscalls.c?h=v6.19#n132)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sched/syscalls.c?h=v6.18.18#n132)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n382)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn nice(increment: Int) -> Long {
    // SAFETY: `nice` takes a single integer argument and has no pointer
    // preconditions.
    unsafe { syscall1(Sysno::Nice, increment as isize) as Long }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::nice;

    #[test]
    fn test_nice_zero_increment() {
        let result = nice(0 as Int);
        assert_eq!(result, 0, "nice(0) failed: {result}");
    }
}
