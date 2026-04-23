use celer_system_linux_ctypes::Long;

use crate::arch::current::{Sysno, syscall0};

/// Return from a legacy signal handler through the kernel signal frame.
///
/// Linux 1.0 and current x86-32 expose this historical i386 syscall as a
/// zero-argument signal-frame return path. The kernel consumes the signal
/// frame already placed on the user stack by signal delivery.
///
/// # Kernel Support
/// - Introduced: Linux 0.99.8
/// - Behavior changes: Linux 0.99.8 exposed `sys_sigreturn(int, unsigned
///   long, unsigned long)` and restored only the blocked-signal mask; Linux
///   1.0 and current x86-32 kernels use the frame-based zero-argument ABI and
///   restore the saved integer register state as well
/// - Availability: x86 32-bit exposes the legacy `sys_sigreturn` entry point
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel reads the saved signal frame from the interrupted user stack.
/// - Linux 1.0 restores the saved blocked-signal mask and the saved integer
///   register state from that frame.
/// - Linux 1.0 validates the saved segment selectors before restoring them.
/// - Linux 1.0 clears `orig_eax` before resuming, which disables syscall
///   restart checks on the restored path.
/// - The saved `eax` value becomes the resumed return register value.
/// - Linux 1.0 stores but does not restore any x87 state from the frame.
///
/// # Errors
/// - No errno is reachable from the Linux 1.0 `sys_sigreturn` entry path.
/// - Linux 1.0 malformed or unreadable frames exit the task with `SIGSEGV`
///   instead of returning an errno.
/// - Current x86-32 malformed or unreadable frames force `SIGSEGV` and still
///   do not return an errno.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigreturn.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/signal_32.c?h=v7.0#n149)
/// - LTS: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/signal_32.c?h=v7.0#n149)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n217)
///
/// # Historical References
/// - First appearance: [Linux 0.99.8](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.99.8#n146)
/// - Linux 1.0 syscall number:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n128)
/// - Linux 1.0 syscall table:
///   [kernel/sched.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n142)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn sigreturn() -> Long {
    syscall0(Sysno::Sigreturn) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, PidT};

    use crate::sys::{exit, fork, waitpid};

    use super::sigreturn;
    use crate::arch::current::Sysno;

    const SIGSEGV: Int = 11;

    #[test]
    fn test_sigreturn_sysno() {
        assert_eq!(Sysno::Sigreturn as isize, 119);
    }

    #[test]
    fn test_sigreturn_without_signal_frame_sigsegvs() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");

        fn exercise_sigreturn_in_child(pid: PidT) {
            if pid == 0 {
                let _ = sigreturn();
                exit(1);
            }
        }

        exercise_sigreturn_in_child(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(
            status & 0x7f,
            SIGSEGV,
            "sigreturn without a kernel signal frame should deliver SIGSEGV"
        );
    }
}
