use celer_system_linux_ctypes::{Long, OldSigsetT};

use crate::arch::current::{Sysno, syscall3};

/// Atomically replace the legacy blocked-signal mask and wait for a handler.
///
/// This is the historical i386 `sigsuspend` syscall ABI, which passes the new
/// legacy 32-bit mask word in the third argument slot.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 0.12 restored the old mask through an explicit
///   restart path in `sys_sigsuspend`; Linux 1.0 and current x86 kernels keep
///   the same three-argument ABI but route mask restoration through the
///   signal-frame return path
/// - Availability: x86 32-bit exposes the legacy `sys_sigsuspend` entry point
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `mask` is the historical 32-bit blocked-signal mask word to install
///   while waiting.
/// - The first two syscall arguments are legacy restart bookkeeping slots and
///   are ignored by Linux 1.0 and current x86 kernels.
/// - The kernel does not allow `SIGKILL` or `SIGSTOP` to remain blocked.
/// - The call waits until signal delivery runs a user handler; ignored signals
///   and default-disposition stop/continue cases keep waiting.
/// - After the handler returns through `sigreturn`, the kernel restores the
///   saved blocked-signal mask.
///
/// # Errors
/// - `EINTR`: a signal handler ran and the interrupted `sigsuspend` resumed as
///   an interrupted syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sigsuspend.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4887)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4889)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=1.0#n109)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/signal.c?h=0.12#n48)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n81)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn sigsuspend(mask: OldSigsetT) -> Long {
    // SAFETY: `sigsuspend` takes only scalar arguments and has no Rust-side
    // memory-safety preconditions. The first two slots are the historical
    // restart bookkeeping arguments and are ignored by the x86 ABI.
    unsafe { syscall3(Sysno::Sigsuspend, 0, 0, mask as isize) as Long }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        thread,
        time::Duration,
    };

    use celer_system_linux_ctypes::{Int, Long, OldSigsetT, PidT};

    use crate::sys::{
        SIG_IGN, SigHandler, exit, fork, getpid, kill, sgetmask, sig_handler,
        sig_handler_from_raw, signal, ssetmask, waitpid,
    };

    use super::sigsuspend;

    const EINTR: Long = -4;
    const SIGUSR1: Int = 10;
    const SIGUSR2: Int = 12;
    static SIGUSR2_HANDLED: AtomicBool = AtomicBool::new(false);

    extern "C" fn handle_sigusr1(_sig: Int) {}

    extern "C" fn handle_sigusr2(_sig: Int) {
        SIGUSR2_HANDLED.store(true, Ordering::SeqCst);
    }

    struct RestoreSignal {
        sig: Int,
        old_handler: SigHandler,
    }

    impl Drop for RestoreSignal {
        fn drop(&mut self) {
            let _ = unsafe { signal(self.sig, self.old_handler) };
        }
    }

    struct RestoreMask(OldSigsetT);

    impl Drop for RestoreMask {
        fn drop(&mut self) {
            let _ = ssetmask(self.0);
        }
    }

    fn assert_clean_exit(status: Int) {
        assert_eq!(status & 0x7f, 0, "child should not die from a signal");
        assert_eq!((status >> 8) & 0xff, 0, "child should exit with status 0");
    }
    fn spawn_signal_sender(
        target: PidT,
        first: Int,
        second: Option<(u64, Int)>,
    ) -> PidT {
        let pid = fork();
        assert!(pid >= 0, "fork for signal sender failed: {pid}");

        if pid == 0 {
            thread::sleep(Duration::from_millis(50));
            let rc = kill(target, first);
            if rc != 0 {
                exit(1);
            }

            if let Some((delay_ms, sig)) = second {
                thread::sleep(Duration::from_millis(delay_ms));
                let rc = kill(target, sig);
                if rc != 0 {
                    exit(2);
                }
            }

            exit(0);
        }

        pid
    }

    #[test]
    fn test_sigsuspend_returns_eintr_and_restores_mask() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        fn use_pid(pid: PidT) {
            if pid == 0 {
                let old_handler =
                    unsafe { signal(SIGUSR1, sig_handler(handle_sigusr1)) };
                assert!(
                    old_handler >= 0,
                    "installing SIGUSR1 handler failed: {old_handler}"
                );
                let _restore_signal = RestoreSignal {
                    sig: SIGUSR1,
                    old_handler: sig_handler_from_raw(old_handler),
                };

                let requested_mask =
                    (1 as OldSigsetT) << ((SIGUSR2 as OldSigsetT) - 1);
                let original_mask = ssetmask(requested_mask);
                let _restore_mask = RestoreMask(original_mask);

                let sender = spawn_signal_sender(getpid(), SIGUSR1, None);

                let rc = sigsuspend(0);
                let mut sender_status = 0;
                let waited =
                    unsafe { waitpid(sender, &raw mut sender_status, 0) };

                assert_eq!(waited, sender);
                assert_clean_exit(sender_status);
                assert_eq!(rc, EINTR, "expected EINTR after handled signal");
                assert_eq!(
                    sgetmask(),
                    requested_mask,
                    "sigsuspend should restore the original blocked-signal mask"
                );
                exit(0);
            }
        }

        use_pid(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_clean_exit(status);
    }

    #[test]
    fn test_sigsuspend_ignores_ignored_signal_until_handler_runs() {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        fn use_pid(pid: PidT) {
            if pid == 0 {
                SIGUSR2_HANDLED.store(false, Ordering::SeqCst);

                let old_usr1 = unsafe { signal(SIGUSR1, SIG_IGN) };
                assert!(old_usr1 >= 0, "installing SIG_IGN failed: {old_usr1}");
                let _restore_usr1 = RestoreSignal {
                    sig: SIGUSR1,
                    old_handler: sig_handler_from_raw(old_usr1),
                };

                let old_usr2 =
                    unsafe { signal(SIGUSR2, sig_handler(handle_sigusr2)) };
                assert!(
                    old_usr2 >= 0,
                    "installing SIGUSR2 handler failed: {old_usr2}"
                );
                let _restore_usr2 = RestoreSignal {
                    sig: SIGUSR2,
                    old_handler: sig_handler_from_raw(old_usr2),
                };

                let sender = spawn_signal_sender(
                    getpid(),
                    SIGUSR1,
                    Some((200, SIGUSR2)),
                );

                let rc = sigsuspend(0);

                let mut sender_status = 0;
                let waited =
                    unsafe { waitpid(sender, &raw mut sender_status, 0) };

                assert_eq!(waited, sender);
                assert_clean_exit(sender_status);
                assert_eq!(rc, EINTR, "expected EINTR after handled signal");
                assert!(
                    SIGUSR2_HANDLED.load(Ordering::SeqCst),
                    "sigsuspend should return only after running the SIGUSR2 handler"
                );
                exit(0);
            }
        }

        use_pid(pid);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_clean_exit(status);
    }
}
