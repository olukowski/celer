use celer_system_linux_ctypes::Long;

use crate::sys;

/// Return from a legacy signal handler through the kernel signal frame.
///
/// This wrapper preserves the raw zero-argument ABI and return register value.
/// It has no syscall-specific error enum because no errno return is reachable
/// from this entry path; malformed frames terminate the task instead of
/// returning an error.
///
/// See [`sys::sigreturn`] for kernel behavior, frame requirements, and source
/// references.
///
/// # Safety
/// The current user stack must contain a valid kernel-built legacy signal
/// frame whose saved execution state may be restored by the kernel.
#[cfg(target_arch = "x86")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn sigreturn() -> Long {
    // SAFETY: forwarded from this wrapper's safety contract.
    unsafe { sys::sigreturn() }
}
