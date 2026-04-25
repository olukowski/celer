use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Long, Tms};

use crate::sys;

/// Return process and child CPU time statistics.
///
/// This safe wrapper replaces the nullable raw output pointer with
/// `Option<&mut MaybeUninit<Tms>>` and returns the raw clock tick count.
///
/// On success, when `tbuf` is `Some`, the kernel has initialized that output
/// record. The tick count is returned directly because this syscall can return
/// a negative value on success after wraparound.
///
/// See [`sys::times`] for kernel behavior, reachable errors, and source
/// references.
pub fn times(tbuf: Option<&mut MaybeUninit<Tms>>) -> Long {
    let ptr = tbuf
        .map(|tbuf| tbuf.as_mut_ptr())
        .unwrap_or(core::ptr::null_mut());

    // SAFETY: non-null pointers are writable for one `Tms`; null is an
    // explicitly supported input.
    unsafe { sys::times(ptr) }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::Tms;

    use super::times;

    #[test]
    fn test_times_null_ok() {
        let _ = times(None);
    }

    #[test]
    fn test_times_writes_output() {
        let mut tms = MaybeUninit::<Tms>::uninit();
        let ret = times(Some(&mut tms));

        assert_ne!(ret, -14, "times should not fail with EFAULT: {ret}");
    }
}
