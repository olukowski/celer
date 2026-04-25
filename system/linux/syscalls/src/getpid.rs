use celer_system_linux_ctypes::PidT;

use crate::sys;

/// Return the process ID of the calling process.
///
/// This safe wrapper exposes the raw `getpid(2)` success value directly. The
/// syscall takes no arguments and the verified kernel entry path has no error
/// returns.
///
/// See [`sys::getpid`] for kernel behavior, historical notes, and source
/// references.
pub fn getpid() -> PidT {
    sys::getpid()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::getpid;
    use crate::sys;

    #[test]
    fn test_getpid_matches_raw() {
        assert_eq!(getpid(), sys::getpid());
    }
}
