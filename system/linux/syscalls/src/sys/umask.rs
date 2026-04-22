use celer_system_linux_ctypes::UModeT;

use crate::arch::current::{Sysno, syscall1};

/// Set the calling process's file mode creation mask.
///
/// # Kernel Support
/// - Available in Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel stores `mask & S_IRWXUGO` as the current umask.
/// - The previous umask is returned to the caller.
/// - This syscall has no pointer arguments and no user-memory access.
///
/// # Errors
/// - None
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/umask.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1960)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1960)
///
/// # Historical References
/// - First stable implementation: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n777)
pub fn umask(mask: UModeT) -> UModeT {
    // SAFETY: `umask` takes a single integer argument and has no pointer
    // validity requirements.
    (unsafe { syscall1(Sysno::Umask, mask as isize) }) as UModeT
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use celer_system_linux_ctypes::UModeT;

    use super::umask;

    static UMASK_LOCK: Mutex<()> = Mutex::new(());

    struct RestoreUmask(UModeT);

    impl Drop for RestoreUmask {
        fn drop(&mut self) {
            let _ = umask(self.0);
        }
    }

    #[test]
    fn test_umask() {
        let _guard = UMASK_LOCK.lock().unwrap();

        let original = umask(0);
        let _restore = RestoreUmask(original);

        let previous = umask(0o123 as UModeT);
        assert_eq!(previous, 0, "umask should return the previous mask");

        let previous = umask(!0 as UModeT);
        assert_eq!(previous, 0o123, "umask should return the previous mask");

        let masked = umask(0);
        assert_eq!(masked, 0o777, "umask should mask off non-mode bits");
    }
}
