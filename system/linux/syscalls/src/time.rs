use core::mem::MaybeUninit;

use celer_system_linux_ctypes::TimeT;

use crate::sys;

/// Return the current calendar time in seconds since the Unix epoch.
///
/// This safe wrapper replaces the nullable raw output pointer with
/// `Option<&mut MaybeUninit<TimeT>>`.
///
/// On success, returns the current time. When `tloc` is `Some`, the kernel has
/// also initialized that slot with the same value.
///
/// See [`sys::time`] for kernel behavior, reachable errors, and source
/// references.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn time(tloc: Option<&mut MaybeUninit<TimeT>>) -> TimeT {
    let ptr = tloc
        .map(|tloc| tloc.as_mut_ptr())
        .unwrap_or(core::ptr::null_mut());

    // SAFETY: non-null pointers are writable for one `TimeT`; null is an
    // explicitly supported input.
    unsafe { sys::time(ptr) }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use super::time;

    #[test]
    fn test_time_null_ok() {
        assert!(time(None) > 0);
    }

    #[test]
    fn test_time_writes_output() {
        let mut stored = MaybeUninit::uninit();
        let now = time(Some(&mut stored));

        // SAFETY: `time` initialized `stored`.
        let stored = unsafe { stored.assume_init() };
        assert_eq!(now, stored);
    }
}
