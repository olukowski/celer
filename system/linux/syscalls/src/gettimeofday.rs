use core::{mem::MaybeUninit, ptr};

use celer_system_linux_ctypes::{Timeval, Timezone};

use crate::sys;

/// Get the current time of day and optional kernel timezone state.
///
/// This safe wrapper turns each nullable raw output pointer into an
/// `Option<&mut MaybeUninit<_>>`. Passing `None` for either output preserves
/// the raw syscall's null-pointer behavior.
///
/// On return, the kernel has initialized each provided output slot.
///
/// See [`sys::gettimeofday`] for kernel behavior, historical notes, reachable
/// raw errors, and source references.
pub fn gettimeofday(
    mut tv: Option<&mut MaybeUninit<Timeval>>,
    mut tz: Option<&mut MaybeUninit<Timezone>>,
) {
    let tv = tv.as_mut().map_or(ptr::null_mut(), |tv| tv.as_mut_ptr());
    let tz = tz.as_mut().map_or(ptr::null_mut(), |tz| tz.as_mut_ptr());

    // SAFETY: each non-null pointer comes from a `MaybeUninit` output slot,
    // and null is a meaningful input for both raw syscall arguments. The only
    // raw error path is an inaccessible non-null output pointer, which this
    // wrapper does not expose.
    let _ = unsafe { sys::gettimeofday(tv, tz) };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{Timeval, Timezone};

    use super::gettimeofday;
    use crate::sys;

    #[test]
    fn test_gettimeofday_both_outputs() {
        let mut tv = MaybeUninit::<Timeval>::uninit();
        let mut tz = MaybeUninit::<Timezone>::uninit();

        gettimeofday(Some(&mut tv), Some(&mut tz));

        let tv = unsafe { tv.assume_init() };
        let tz = unsafe { tz.assume_init() };

        assert!(tv.tv_sec > 0, "tv_sec should be a positive epoch value");
        assert!(
            (0..1_000_000).contains(&tv.tv_usec),
            "tv_usec should be in [0, 1_000_000), got {}",
            tv.tv_usec
        );
        assert_ne!(tz.tz_minuteswest, -1);
        assert_ne!(tz.tz_dsttime, -1);
    }

    #[test]
    fn test_gettimeofday_tv_none() {
        let mut tz = MaybeUninit::<Timezone>::uninit();

        gettimeofday(None, Some(&mut tz));
    }

    #[test]
    fn test_gettimeofday_tz_none() {
        let mut tv = MaybeUninit::<Timeval>::uninit();

        gettimeofday(Some(&mut tv), None);
    }

    #[test]
    fn test_gettimeofday_both_none() {
        gettimeofday(None, None);
    }

    #[test]
    fn test_gettimeofday_matches_raw_with_null_outputs() {
        gettimeofday(None, None);
        let raw = unsafe {
            sys::gettimeofday(core::ptr::null_mut(), core::ptr::null_mut())
        };

        assert_eq!(raw, 0);
    }
}
