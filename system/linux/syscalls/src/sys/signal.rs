use core::ptr;

use celer_system_linux_ctypes::{Int, Long};

use crate::arch::current::{Sysno, syscall2};

/// Legacy `sys_signal` handler/disposition word.
pub type SigHandler = *const ();

/// Default signal disposition.
pub const SIG_DFL: SigHandler = ptr::null();

/// Ignore the delivered signal.
pub const SIG_IGN: SigHandler = ptr::without_provenance::<()>(1);

/// Convert a legacy `signal` raw return value back into a handler word.
///
/// `raw` must be a nonnegative handler/disposition value previously returned
/// by a successful call to [`signal`].
pub fn sig_handler_from_raw(raw: Long) -> SigHandler {
    debug_assert!(raw >= 0);
    ptr::without_provenance::<()>(raw as usize)
}

/// Convert a C-compatible signal handler function into a legacy handler word.
pub fn sig_handler(handler: extern "C" fn(Int)) -> SigHandler {
    handler as SigHandler
}

/// Install a legacy signal handler for `sig`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 rejected handler addresses greater than or
///   equal to `TASK_SIZE` with `EFAULT`; current kernels no longer perform
///   that front-end address check in the `sys_signal` entry path.
/// - Availability: x86 32-bit exposes the legacy `sys_signal` entry point
///   for backward compatibility
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - This is the old BSD-style signal API, superseded by `sigaction`.
/// - The kernel installs the handler with `SA_ONESHOT | SA_NOMASK`.
/// - On success, the previous handler is returned.
///
/// # Errors
/// - `EINVAL`: `sig` is not a valid signal number, or the target disposition
///   is immutable.
/// - `EFAULT`: on Linux 1.0, `handler` is greater than or equal to the
///   process `TASK_SIZE` limit.
///
/// # Safety
/// - `handler` must be a signal disposition value accepted by the kernel for
///   this ABI. If it names a user handler, that handler must remain valid for
///   signal delivery and use the correct ABI.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/signal.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4806)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4806)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.10#n48)
pub unsafe fn signal(sig: Int, handler: SigHandler) -> Long {
    // SAFETY: the caller must uphold the ABI and lifetime requirements for any
    // installed signal handler address.
    unsafe {
        syscall2(Sysno::Signal, sig as isize, handler.addr() as isize) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::{SIG_DFL, signal};

    #[test]
    fn test_signal_invalid_sig() {
        let result = unsafe { signal(0 as Int, SIG_DFL) };

        assert!(result < 0, "signal should have failed: {result}");
    }
}
