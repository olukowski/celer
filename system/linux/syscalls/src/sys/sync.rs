use celer_system_linux_ctypes::Long;

use crate::arch::current::{Sysno, syscall0};

/// Flush all pending filesystem data to disk.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: historical implementation moved from legacy buffer
///   syncing to the modern writeback path
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel wakes flusher threads, syncs filesystem metadata and data, and
///   then syncs block devices before returning.
/// - The syscall returns `0` on success.
///
/// # Errors
/// - Never fails in the current kernel implementation.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sync.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/sync.c?h=v6.19#n111)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/sync.c?h=v6.18.18#n111)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/buffer.c?h=1.0#n166)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/buffer.c?h=0.10#n44)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn sync() -> Long {
    // SAFETY: `sync` takes no arguments and has no caller-side preconditions.
    syscall0(Sysno::Sync) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::sync;

    #[test]
    fn test_sync() {
        assert_eq!(sync(), 0, "sync failed");
    }
}
