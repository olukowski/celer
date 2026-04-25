use celer_system_linux_ctypes::OldUidT;

use crate::sys;

/// Return the caller's effective user ID through the legacy x86 `geteuid16` ABI.
///
/// This wrapper keeps the raw syscall's no-argument shape and returns the
/// legacy 16-bit user ID directly.
///
/// See [`sys::geteuid16`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - Never fails.
pub fn geteuid16() -> OldUidT {
    sys::geteuid16()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldUidT;

    use crate::sys;

    use super::geteuid16;

    #[test]
    fn test_geteuid16_matches_raw() {
        let wrapped = geteuid16();
        let raw = sys::geteuid16();

        assert_eq!(wrapped, raw, "wrapped geteuid16 should match raw syscall");
    }

    #[test]
    fn test_geteuid16_type() {
        let _: OldUidT = geteuid16();
    }
}
