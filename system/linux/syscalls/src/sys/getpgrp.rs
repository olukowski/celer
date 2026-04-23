use celer_system_linux_ctypes::PidT;

use crate::arch::current::{Sysno, syscall0};

/// Returns the process group ID (PGID) of the calling process.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: current kernels implement `getpgrp` via `do_getpgid(0)`
///   instead of a direct `current->pgrp` field read.
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Errors
/// - Never fails (no error conditions)
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getpgrp.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1222)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1222)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n522)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n201)
pub fn getpgrp() -> PidT {
    // SAFETY: `getpgrp` is always safe to call.
    syscall0(Sysno::Getpgrp) as PidT
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::getpgrp;

    #[test]
    fn test_getpgrp() {
        let pgid = getpgrp();

        assert!(
            pgid > 0,
            "getpgrp should return a positive process group ID"
        );
    }
}
