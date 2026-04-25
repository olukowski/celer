use celer_system_linux_ctypes::OldUidT;

use crate::sys;

/// Return the caller's real user ID through the legacy x86 `getuid16` ABI.
///
/// This wrapper keeps the raw syscall's no-argument shape and returns the
/// legacy 16-bit user ID directly.
///
/// See [`sys::getuid16`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - Never fails.
pub fn getuid16() -> OldUidT {
    sys::getuid16()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldUidT;

    use super::getuid16;
    use crate::sys;

    #[test]
    fn test_getuid16_matches_raw() {
        let wrapped = getuid16();
        let raw = sys::getuid16();

        assert_eq!(wrapped, raw, "wrapped getuid16 should match raw syscall");
    }

    #[test]
    fn test_getuid16_type() {
        let _: OldUidT = getuid16();
    }
}
