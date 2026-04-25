use celer_system_linux_ctypes::PidT;

use crate::sys;

/// Return the process group ID of the calling process.
///
/// This safe wrapper exposes the raw `getpgrp(2)` success value directly. The
/// syscall takes no arguments and the verified kernel entry path has no error
/// returns.
///
/// See [`sys::getpgrp`] for kernel behavior, historical notes, and source
/// references.
pub fn getpgrp() -> PidT {
    sys::getpgrp()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::getpgrp;
    use crate::sys;

    #[test]
    fn test_getpgrp_matches_raw() {
        assert_eq!(getpgrp(), sys::getpgrp());
    }
}
