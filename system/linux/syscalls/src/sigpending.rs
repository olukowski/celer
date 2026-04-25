use core::mem::MaybeUninit;

use celer_system_linux_ctypes::OldSigsetT;

use crate::sys;

/// Return the caller's pending blocked-signal mask word.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldSigsetT>`.
///
/// On return, the kernel has initialized `set` with the legacy pending
/// blocked-signal mask word.
///
/// See [`sys::sigpending`] for kernel behavior, reachable raw errors, and
/// source references.
#[cfg(target_arch = "x86")]
pub fn sigpending(set: &mut MaybeUninit<OldSigsetT>) {
    // SAFETY: `MaybeUninit<OldSigsetT>` provides writable storage for one
    // legacy signal mask word. The only raw error path is an inaccessible
    // output pointer, which this wrapper does not expose.
    let _ = unsafe { sys::sigpending(set.as_mut_ptr()) };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use super::sigpending;

    #[test]
    fn test_sigpending_ok() {
        let mut pending = MaybeUninit::uninit();

        sigpending(&mut pending);
    }
}
