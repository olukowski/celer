use celer_system_linux_ctypes::{Int, Long};

use crate::arch::current::{Sysno, syscall2};

/// Install a legacy signal handler for `sig`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
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
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/signal.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4806)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4806)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n63)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.10#n48)
pub fn signal(sig: Int, handler: usize) -> Long {
    // SAFETY: `signal` takes only scalar arguments and has no Rust-side safety
    // preconditions.
    unsafe { syscall2(Sysno::Signal, sig as isize, handler as isize) as Long }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Int;

    use super::signal;

    #[test]
    fn test_signal_invalid_sig() {
        let result = signal(0 as Int, 0);

        assert!(result < 0, "signal should have failed: {result}");
    }
}
