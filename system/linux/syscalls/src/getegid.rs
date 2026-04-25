use celer_system_linux_ctypes::OldGidT;

use crate::sys;

/// Return the caller's effective group ID through the legacy x86 `getegid16` ABI.
///
/// This wrapper keeps the raw syscall's no-argument shape and returns the
/// legacy 16-bit group ID directly.
///
/// See [`sys::getegid16`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - Never fails.
pub fn getegid16() -> OldGidT {
    sys::getegid16()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldGidT;

    use crate::sys;

    use super::getegid16;

    #[test]
    fn test_getegid16_matches_raw() {
        let wrapped = getegid16();
        let raw = sys::getegid16();

        assert_eq!(wrapped, raw, "wrapped getegid16 should match raw syscall");
    }

    #[test]
    fn test_getegid16_type() {
        let _: OldGidT = getegid16();
    }
}
