use celer_system_linux_ctypes::OldUidT;

use crate::arch::current::{Sysno, syscall0};

/// Return the effective user ID through the legacy i386 `geteuid16` ABI.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: modern i386 still uses the legacy 16-bit `geteuid`
///   entry (`sys_geteuid16`)
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Returns the caller's effective user ID mapped into the caller's user
///   namespace.
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/geteuid.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n208)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n208)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n743)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n367)
pub fn geteuid16() -> OldUidT {
    // SAFETY: `geteuid16` takes no arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall0(Sysno::Geteuid) as OldUidT }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::arch::current::{Sysno, syscall0};

    use super::geteuid16;
    use celer_system_linux_ctypes::OldUidT;

    #[test]
    fn test_geteuid() {
        let euid = geteuid16();
        let expected = unsafe { syscall0(Sysno::Geteuid) as OldUidT };

        assert_eq!(euid, expected, "geteuid16 should match the raw syscall");
    }
}
