use celer_system_linux_ctypes::UModeT;

use crate::sys;

/// Set the calling process's file mode creation mask.
///
/// This safe wrapper preserves the raw integer argument and returns the
/// previous mask.
///
/// See [`sys::umask`] for kernel behavior and source references.
pub fn umask(mask: UModeT) -> UModeT {
    sys::umask(mask)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::UModeT;

    use crate::sys::test_support::process_global_state_guard;

    use super::umask;

    struct RestoreUmask(UModeT);

    impl Drop for RestoreUmask {
        fn drop(&mut self) {
            let _ = umask(self.0);
        }
    }

    #[test]
    fn test_umask_returns_previous_mask() {
        let _guard = process_global_state_guard();
        let original = umask(0);
        let _restore = RestoreUmask(original);

        assert_eq!(umask(0o123 as UModeT), 0);
        assert_eq!(umask(!0 as UModeT), 0o123);
        assert_eq!(umask(0), 0o777);
    }
}
