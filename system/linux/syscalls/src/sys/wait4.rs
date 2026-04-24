use celer_system_linux_ctypes::{Int, PidT, Rusage};

use crate::arch::current::{Sysno, syscall4};

/// Wait for a child selected by `pid` and optionally collect its wait status
/// and resource usage.
///
/// This wrapper targets the original Linux 1.0 `wait4` entry point at x86
/// syscall slot `114`.
///
/// # Safety
/// - `stat_addr`, when non-null, must be valid to write one [`Int`] value for
///   the duration of the syscall.
/// - `ru`, when non-null, must be valid to write one [`Rusage`] value for the
///   duration of the syscall.
/// - If `stat_addr` or `ru` is non-null, the pointed-to memory must not
///   violate Rust's aliasing rules while the kernel may write through it.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepted only `WNOHANG`, `WUNTRACED`, and
///   `__WCLONE` without rejecting unknown option bits; current kernels also
///   accept `WCONTINUED`, `__WNOTHREAD`, and `__WALL`, reject other bits with
///   `EINVAL`, reject `pid == INT_MIN` with `ESRCH`, and surface `ru`
///   copy-out faults as `EFAULT`
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `pid > 0` waits for that exact child PID.
/// - `pid == 0` waits for any child in the caller's process group.
/// - `pid == -1` waits for any child.
/// - `pid < -1` waits for any child in process group `-pid`.
/// - If `stat_addr` is non-null and a child is reported, the kernel stores the
///   encoded wait status there.
/// - If `ru` is non-null and a child is reported, the kernel stores child
///   resource usage there.
/// - Returns the reported child PID on success, or `0` when `WNOHANG` is set
///   and no matching child is waitable.
/// - Linux 1.0 called `getrusage(..., RUSAGE_BOTH, ru)` for non-null `ru` but
///   ignored that helper's error return; current kernels instead return
///   `EFAULT` if copying the completed `rusage` record back to user memory
///   fails.
///
/// # Errors
/// - `ECHILD`: No child matching `pid` and `options` exists or remains
///   waitable.
/// - `EFAULT`: On Linux 1.0, `stat_addr` is non-null and not writable for one
///   [`Int`] value before the wait begins; on current kernels, a child has
///   already been reported and either the `stat_addr` or `ru` copy-out fails.
/// - `EINVAL`: On current kernels, `options` contains unsupported bits.
/// - `ESRCH`: On current kernels, `pid == INT_MIN`.
/// - `EINTR`: The blocking wait was interrupted by a signal before a child
///   became waitable.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/wait4.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/exit.c?h=v6.19#n1899)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/exit.c?h=v6.18.18#n1894)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/exit.c?h=1.0#n484)
///
/// # Historical References
/// - Linux 1.0 wait flags:
///   [include/linux/wait.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/wait.h?h=1.0#n4)
/// - Linux 1.0 `struct rusage`:
///   [include/linux/resource.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/resource.h?h=1.0#n19)
pub unsafe fn wait4(
    pid: PidT,
    stat_addr: *mut Int,
    options: Int,
    ru: *mut Rusage,
) -> PidT {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall4(
            Sysno::Wait4,
            pid as isize,
            stat_addr.addr() as isize,
            options as isize,
            ru.addr() as isize,
        ) as PidT
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    use celer_system_linux_ctypes::{Int, Rusage, Timeval};

    use crate::arch::current::Sysno;
    #[cfg(target_arch = "aarch64")]
    use crate::sys::test_support::signal as c_signal;
    #[cfg(target_arch = "x86")]
    use crate::sys::{SigHandler, sig_handler, sig_handler_from_raw, signal};
    use crate::sys::{
        getpid, kill,
        test_support::{_exit as exit, fork, pause, waitpid},
    };

    use super::wait4;

    const EINTR: Int = -(4 as Int);
    const WNOHANG: Int = 1;
    const SIGKILL: Int = 9;
    const SIGUSR1: Int = 10;

    static WAIT4_SIGNAL_LOCK: Mutex<()> = Mutex::new(());

    extern "C" fn handle_sigalrm(_: Int) {}

    #[cfg(target_arch = "aarch64")]
    type SigHandler = libc::sighandler_t;

    #[cfg(target_arch = "aarch64")]
    fn sig_handler(handler: extern "C" fn(Int)) -> SigHandler {
        handler as usize
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn signal(sig: Int, handler: SigHandler) -> isize {
        unsafe { c_signal(sig, handler) as isize }
    }

    #[cfg(target_arch = "aarch64")]
    fn sig_handler_from_raw(raw: isize) -> SigHandler {
        raw as usize
    }

    struct RestoreHandler {
        sig: Int,
        old: SigHandler,
    }

    impl Drop for RestoreHandler {
        fn drop(&mut self) {
            let _ = unsafe { signal(self.sig, self.old) };
        }
    }

    fn spawn_signal_sender(target: Int, sig: Int) -> Int {
        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {pid}");
        if pid == 0 {
            thread::sleep(Duration::from_millis(100));
            let rc = kill(target, sig);
            if rc != 0 {
                unsafe { exit(1) };
            }
            unsafe { exit(0) };
        }

        pid
    }

    fn assert_clean_exit(status: Int) {
        assert_eq!(
            status & 0x7f,
            0,
            "expected normal exit status, got {status}"
        );
        assert_eq!(
            (status >> 8) & 0xff,
            0,
            "expected zero exit code, got {status}"
        );
    }

    fn sentinel_rusage() -> Rusage {
        Rusage {
            ru_utime: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
            ru_stime: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
            ru_maxrss: -1,
            ru_ixrss: -1,
            ru_idrss: -1,
            ru_isrss: -1,
            ru_minflt: -1,
            ru_majflt: -1,
            ru_nswap: -1,
            ru_inblock: -1,
            ru_oublock: -1,
            ru_msgsnd: -1,
            ru_msgrcv: -1,
            ru_nsignals: -1,
            ru_nvcsw: -1,
            ru_nivcsw: -1,
        }
    }

    fn zeroed_rusage() -> Rusage {
        Rusage {
            ru_utime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        }
    }

    #[test]
    fn test_wait4_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Wait4 as isize, 114);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Wait4 as isize, 260);
    }

    #[test]
    fn test_wait4_reaps_exited_child_and_collects_rusage() {
        let pid = unsafe { fork() };
        if pid == 0 {
            unsafe { exit(0) };
        }

        let mut status: Int = -1;
        let mut usage = sentinel_rusage();
        // SAFETY: both outputs are valid and uniquely writable for the call.
        let waited = unsafe { wait4(pid, &raw mut status, 0, &raw mut usage) };

        assert_eq!(waited, pid, "wait4 failed: {waited}");
        assert_eq!(
            status & 0x7f,
            0,
            "expected normal exit status, got {status}"
        );
        assert_ne!(usage, sentinel_rusage(), "wait4 should populate rusage");
    }

    #[test]
    fn test_wait4_wnohang_returns_zero_for_live_child() {
        let pid = unsafe { fork() };
        if pid == 0 {
            let _ = unsafe { pause() };
            unsafe { exit(0) };
        }

        let mut status: Int = -1;
        let mut usage = zeroed_rusage();
        // SAFETY: both outputs are valid and uniquely writable for the call.
        let waited =
            unsafe { wait4(pid, &raw mut status, WNOHANG, &raw mut usage) };

        assert_eq!(waited, 0, "expected WNOHANG to return 0, got {waited}");
        assert_eq!(status, -1, "WNOHANG should not have written a status");

        assert_eq!(kill(pid, SIGKILL), 0, "kill(SIGKILL) failed");
        // SAFETY: both outputs are valid and uniquely writable for the call.
        let reaped = unsafe { wait4(pid, &raw mut status, 0, &raw mut usage) };
        assert_eq!(reaped, pid, "cleanup wait4 failed: {reaped}");
    }

    #[test]
    fn test_wait4_returns_echild_without_children() {
        // SAFETY: null output pointers are permitted by the syscall ABI.
        let rc = unsafe {
            wait4(-1, core::ptr::null_mut(), 0, core::ptr::null_mut())
        };

        assert_eq!(rc, -10, "expected ECHILD, got {rc}");
    }

    #[test]
    fn test_wait4_rejects_invalid_options() {
        // SAFETY: null output pointers are permitted by the syscall ABI.
        let rc = unsafe {
            wait4(-1, core::ptr::null_mut(), -1, core::ptr::null_mut())
        };

        assert_eq!(rc, -22, "expected EINVAL for invalid options, got {rc}");
    }

    #[test]
    fn test_wait4_reports_efault_for_bad_rusage_pointer() {
        let pid = unsafe { fork() };
        if pid == 0 {
            unsafe { exit(0) };
        }

        let mut status: Int = 0;
        // SAFETY: this intentionally passes an invalid `ru` pointer to verify
        // the kernel reports `EFAULT` instead of causing Rust-side UB.
        let bad_usage = core::ptr::without_provenance_mut::<Rusage>(usize::MAX);
        let rc = unsafe { wait4(pid, &raw mut status, 0, bad_usage) };

        assert_eq!(rc, -14, "expected EFAULT for bad rusage pointer, got {rc}");

        // SAFETY: cleanup uses null output pointers, which are permitted.
        let _ = unsafe {
            wait4(pid, core::ptr::null_mut(), WNOHANG, core::ptr::null_mut())
        };
    }

    #[test]
    fn test_wait4_reports_efault_for_bad_status_pointer() {
        let pid = unsafe { fork() };
        if pid == 0 {
            unsafe { exit(0) };
        }

        let mut usage = zeroed_rusage();
        // SAFETY: this intentionally passes an invalid `stat_addr` pointer to
        // verify the kernel reports `EFAULT`.
        let bad_status = core::ptr::without_provenance_mut::<Int>(usize::MAX);
        let rc = unsafe { wait4(pid, bad_status, 0, &raw mut usage) };

        assert_eq!(rc, -14, "expected EFAULT for bad status pointer, got {rc}");

        // SAFETY: cleanup uses null output pointers, which are permitted.
        let _ = unsafe {
            wait4(pid, core::ptr::null_mut(), WNOHANG, core::ptr::null_mut())
        };
    }

    #[test]
    fn test_wait4_int_min_pid_returns_esrch() {
        // SAFETY: null output pointers are permitted by the syscall ABI.
        let rc = unsafe {
            wait4(Int::MIN, core::ptr::null_mut(), 0, core::ptr::null_mut())
        };

        assert_eq!(rc, -3, "expected ESRCH for pid == INT_MIN, got {rc}");
    }

    #[test]
    fn test_wait4_interrupted_by_signal_returns_eintr() {
        let _guard = WAIT4_SIGNAL_LOCK.lock().unwrap();

        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {pid}");
        if pid == 0 {
            let old = unsafe { signal(SIGUSR1, sig_handler(handle_sigalrm)) };
            assert_ne!(old, -1, "installing SIGUSR1 handler failed");
            let _restore = RestoreHandler {
                sig: SIGUSR1,
                old: sig_handler_from_raw(old),
            };

            let child = unsafe { fork() };
            assert!(child >= 0, "fork failed: {child}");
            if child == 0 {
                let _ = unsafe { pause() };
                unsafe { exit(0) };
            }

            let sender = spawn_signal_sender(getpid(), SIGUSR1);

            // SAFETY: null output pointers are permitted by the syscall ABI.
            let rc = unsafe {
                wait4(child, core::ptr::null_mut(), 0, core::ptr::null_mut())
            };

            let mut sender_status = 0;
            let waited_sender =
                unsafe { waitpid(sender, &raw mut sender_status, 0) };
            assert_eq!(
                waited_sender, sender,
                "waitpid failed: {waited_sender}"
            );
            assert_clean_exit(sender_status);
            assert_eq!(rc, EINTR, "expected EINTR, got {rc}");

            assert_eq!(kill(child, SIGKILL), 0, "kill(SIGKILL) failed");
            // SAFETY: null output pointers are permitted by the syscall ABI.
            let reaped = unsafe {
                wait4(child, core::ptr::null_mut(), 0, core::ptr::null_mut())
            };
            assert_eq!(reaped, child, "cleanup wait4 failed: {reaped}");
            unsafe { exit(0) };
        }

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid, "waitpid failed: {waited}");
        assert_clean_exit(status);
    }
}
