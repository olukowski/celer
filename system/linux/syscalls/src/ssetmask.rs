use celer_system_linux_ctypes::OldSigsetT;

use crate::sys;

/// Replace the caller's legacy blocked-signal mask word.
///
/// This safe wrapper keeps the scalar mask argument and preserves the raw
/// returned previous mask word. On current kernels that do not configure this
/// legacy entry point, the returned bits are the raw `-ENOSYS` value described
/// by [`sys::ssetmask`].
///
/// On success, returns the previous legacy blocked-signal mask word.
///
/// See [`sys::ssetmask`] for kernel behavior, reachable errors, and source
/// references.
#[cfg(target_arch = "x86")]
pub fn ssetmask(newmask: OldSigsetT) -> OldSigsetT {
    sys::ssetmask(newmask)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldSigsetT;

    use super::ssetmask;
    use crate::sys::test_support::process_global_state_guard;

    const ENOSYS_BITS: OldSigsetT = (-38_i32) as OldSigsetT;

    struct RestoreMask(OldSigsetT);

    impl Drop for RestoreMask {
        fn drop(&mut self) {
            let _ = ssetmask(self.0);
        }
    }

    #[test]
    fn test_ssetmask_smoke() {
        let _guard = process_global_state_guard();

        let _ = ssetmask(0);
    }

    #[test]
    fn test_ssetmask_preserves_errno_shaped_mask_word() {
        let _guard = process_global_state_guard();

        let original = ssetmask(0);
        if original == ENOSYS_BITS {
            return;
        }
        let _restore = RestoreMask(original);

        let requested = (-4095_i32) as OldSigsetT;
        let _ = ssetmask(requested);
        let effective = ssetmask(0);

        assert_ne!(effective, ENOSYS_BITS);
        assert_ne!(
            effective & ((1 as OldSigsetT) << 31),
            0,
            "effective mask should preserve high errno-shaped data bits"
        );
    }
}
