use celer_system_linux_ctypes::OldSigsetT;

use crate::arch::current::{Sysno, syscall0};

/// Return the caller's legacy blocked-signal mask word.
///
/// This is the obsolete i386 `sgetmask` compatibility syscall superseded by
/// `sigprocmask`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 0.10 and Linux 1.0 returned `current->blocked`
///   directly, while current kernels return `current->blocked.sig[0]` and
///   gate the syscall behind `CONFIG_SGETMASK_SYSCALL`
/// - Availability: this wrapper targets the i386 ABI; current kernels also
///   retain `sgetmask` on other legacy arch ABIs, subject to
///   `CONFIG_SGETMASK_SYSCALL`
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On supported kernels, returns the low 32-bit blocked-signal mask word
///   for the calling task.
/// - This wrapper exposes the raw register value as [`OldSigsetT`].
/// - Current kernels built without `CONFIG_SGETMASK_SYSCALL` route the call to
///   `sys_ni_syscall`, so the raw return bits encode `-ENOSYS` instead of a
///   signal mask.
///
/// # Errors
/// - `ENOSYS`: current kernels built without `CONFIG_SGETMASK_SYSCALL`
///   dispatch `sgetmask` to `sys_ni_syscall`; because this wrapper returns the
///   raw mask word, callers observe the raw register bits for `-ENOSYS`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigprocmask.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4784)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4783)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n83)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.10#n15)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n77)
pub fn sgetmask() -> OldSigsetT {
    syscall0(Sysno::Sgetmask) as OldSigsetT
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::OldSigsetT;

    use crate::arch::current::{Sysno, syscall0};

    use super::sgetmask;

    #[test]
    fn test_sgetmask_matches_raw_syscall() {
        let mask = sgetmask();
        let expected = syscall0(Sysno::Sgetmask) as OldSigsetT;

        assert_eq!(mask, expected, "sgetmask should match the raw syscall");
    }
}
