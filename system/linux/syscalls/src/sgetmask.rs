use celer_system_linux_ctypes::OldSigsetT;

use crate::sys;

/// Return the caller's legacy blocked-signal mask word.
///
/// This safe wrapper preserves the raw returned mask word. On current kernels
/// that do not configure this legacy entry point, the returned bits are the raw
/// `-ENOSYS` value described by [`sys::sgetmask`].
///
/// See [`sys::sgetmask`] for kernel behavior, reachable errors, and source
/// references.
#[cfg(target_arch = "x86")]
pub fn sgetmask() -> OldSigsetT {
    sys::sgetmask()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::sgetmask;

    #[test]
    fn test_sgetmask_smoke() {
        let _ = sgetmask();
    }
}
