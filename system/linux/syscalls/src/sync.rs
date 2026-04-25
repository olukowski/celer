use crate::sys;

/// Flush all pending filesystem data to disk.
///
/// This safe wrapper exposes the infallible syscall as `()`.
///
/// See [`sys::sync`] for kernel behavior and source references.
pub fn sync() {
    let _ = sys::sync();
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::sync;

    #[test]
    fn test_sync_returns_unit() {
        assert_eq!(sync(), ());
    }
}
