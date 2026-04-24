use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall0};

/// Historical i386 scheduler-yield hint syscall.
///
/// Linux 1.0 implemented `sys_idle` as a tiny helper that only set the global
/// `need_resched` flag and returned success. Current x86-32 syscall tables
/// still reserve the historical syscall number, but leave it unimplemented so
/// it resolves to `sys_ni_syscall`; x86-64 has no `idle` syscall number.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: current x86-32 kernels keep the syscall number but
///   route it to `sys_ni_syscall`, returning `ENOSYS`
/// - Availability: present on Linux 1.0 x86; current x86-32 kernels still
///   reserve the number but do not implement the historical behavior; absent
///   from current x86-64 syscall tables
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Linux 1.0 takes no arguments.
/// - Linux 1.0 sets `need_resched = 1` and returns `0`.
/// - Current x86-32 kernels return `ENOSYS` from `sys_ni_syscall`.
///
/// # Errors
/// - Linux 1.0: no errno is reachable from the `sys_idle` entry path.
/// - Current x86-32: `ENOSYS`, because the syscall table slot has no native
///   implementation and resolves to `sys_ni_syscall`.
///
/// # References
/// - Linux 1.0 syscall number table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n121)
/// - Linux 1.0 syscall table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n141)
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/swap.c?h=1.0#n272)
/// - Current x86-32 syscall table:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n127)
/// - Current syscall table generator fallback:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/scripts/syscalltbl.sh?h=v6.19#n85)
/// - LTS x86-32 syscall table:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n127)
///
/// # Historical References
/// - Linux 0.10 syscall table range showing no `idle` entry:
///   [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/unistd.h?h=0.10#n60)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn idle() -> Int {
    // SAFETY: `idle` takes no arguments and has no caller-visible memory-
    // safety preconditions.
    unsafe { syscall0(Sysno::Idle) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::idle;
    use crate::arch::current::{Sysno, syscall0};

    const ENOSYS: Int = -(38 as Int);

    #[test]
    fn test_idle_sysno() {
        assert_eq!(Sysno::Idle as isize, 112);
    }

    #[test]
    fn test_idle_matches_raw_syscall() {
        let wrapped = idle();
        let raw = unsafe { syscall0(Sysno::Idle) as Int };

        assert_eq!(wrapped, raw, "idle wrapper should match raw syscall");
        assert_eq!(
            wrapped, ENOSYS,
            "idle should be unimplemented on current x86"
        );
    }
}
