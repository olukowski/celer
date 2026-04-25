use celer_system_linux_ctypes::PidT;

use crate::sys;

/// Return the parent process ID of the calling process.
///
/// This safe wrapper exposes the raw `getppid(2)` success value directly. The
/// syscall takes no arguments and the verified kernel entry path has no error
/// returns.
///
/// See [`sys::getppid`] for kernel behavior, historical notes, and source
/// references.
pub fn getppid() -> PidT {
    sys::getppid()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::getppid;
    use crate::sys;

    #[test]
    fn test_getppid_matches_raw() {
        assert_eq!(getppid(), sys::getppid());
    }
}
