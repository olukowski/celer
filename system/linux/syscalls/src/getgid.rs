use celer_system_linux_ctypes::OldGidT;

use crate::sys;

/// Return the caller's real group ID through the legacy x86 `getgid16` ABI.
///
/// This wrapper keeps the raw syscall's no-argument shape and returns the
/// legacy 16-bit group ID directly.
///
/// See [`sys::getgid16`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - Never fails.
pub fn getgid16() -> OldGidT {
    sys::getgid16()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::OldGidT;

    use crate::sys;

    use super::getgid16;

    #[test]
    fn test_getgid16_matches_raw() {
        let wrapped = getgid16();
        let raw = sys::getgid16();

        assert_eq!(wrapped, raw, "wrapped getgid16 should match raw syscall");
    }

    #[test]
    fn test_getgid16_type() {
        let _: OldGidT = getgid16();
    }
}
