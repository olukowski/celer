use celer_system_linux_ctypes::{FdSet, Int, Timeval, UnsignedLong};

use crate::arch::current::{Sysno, syscall1};

#[repr(C)]
struct SelectArgs {
    nfds: UnsignedLong,
    readfds: UnsignedLong,
    writefds: UnsignedLong,
    exceptfds: UnsignedLong,
    timeout: UnsignedLong,
}

/// Wait for readiness changes on up to `nfds` file descriptors.
///
/// This wrapper targets the original Linux 1.0 x86 syscall slot 82 ABI.
/// The kernel entry takes a single pointer to a packed five-word argument
/// block; this wrapper builds that block internally and exposes the logical
/// `select(nfds, readfds, writefds, exceptfds, timeout)` interface.
///
/// The packed argument block is still the current i386 ABI at syscall slot 82,
/// but the pointed-to `fd_set` layout is not frozen to the Linux 1.0 shape.
/// Linux 1.0 used a 256-bit `fd_set`; current x86 kernels use the current
/// kernel `fd_set` definition, which is 1024 bits.
///
/// # Safety
/// - `readfds`, when non-null, must be valid for the kernel to read and then
///   write one [`FdSet`] value for the duration of the syscall.
/// - `writefds`, when non-null, must be valid for the kernel to read and then
///   write one [`FdSet`] value for the duration of the syscall.
/// - `exceptfds`, when non-null, must be valid for the kernel to read and then
///   write one [`FdSet`] value for the duration of the syscall.
/// - `timeout`, when non-null, must be valid for the kernel to read and then
///   write one [`Timeval`] value for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes:
///   - Linux 1.0 clipped `nfds` to `NR_OPEN` (`256`) and used a 256-bit
///     `fd_set`.
///   - Current i386 kernels still expose syscall slot 82 through the packed
///     `old_select` ABI, but they clip `nfds` to the calling task's current
///     `max_fds` and use the current 1024-bit kernel `fd_set`.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `readfds`, `writefds`, `exceptfds`, and `timeout` may each be null
///   independently.
/// - On success, returns the total number of ready descriptor bits across all
///   three result sets.
/// - On success, each non-null descriptor set is overwritten so that the words
///   covered by `nfds` keep only the ready bits requested for that set.
/// - On Linux 1.0, `nfds < 0` fails with `EINVAL`.
/// - On Linux 1.0, `nfds > 256` is clipped to `256` instead of being rejected.
/// - On current kernels, positive `nfds` values larger than the current
///   `max_fds` snapshot are clipped to that snapshot instead of being
///   rejected.
/// - On current kernels, positive `timeout.tv_usec` overflow is normalized
///   into whole seconds before validation in the `old_select` entry path, so
///   values such as `1_000_001` are accepted instead of failing with
///   `EINVAL`.
/// - If `timeout` is non-null, the kernel writes the remaining timeout back
///   before returning, including `0` on expiry.
///
/// # Errors
/// - `EBADF`: One of the requested descriptors is not open or has no inode.
/// - `EFAULT`: A non-null descriptor-set pointer or non-null `timeout`
///   pointer is not accessible to the kernel.
/// - `EINVAL`: `nfds` is negative.
/// - `EINVAL`: on current kernels, `timeout` contains invalid field values.
/// - `ENOMEM`: Kernel allocation of the temporary select wait table failed.
/// - `EINTR`: on current kernels, no descriptor became ready before return
///   and an unblocked signal interrupted the wait.
///
/// # References
/// - Stable entry:
///   [v7.0 old_select](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/select.c?h=v7.0#n824)
/// - Stable helper:
///   [v7.0 core_sys_select](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/select.c?h=v7.0#n621)
/// - LTS entry:
///   [v6.18.18 old_select](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/select.c?h=v6.18.18#n828)
/// - LTS helper:
///   [v6.18.18 core_sys_select](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/select.c?h=v6.18.18#n621)
/// - First stable:
///   [Linux 1.0 sys_select](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/select.c?h=1.0#n195)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.12 sys_select](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/select.c?h=0.12#n216)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n91)
/// - Linux 1.0 `fd_set` layout:
///   [include/linux/types.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/types.h?h=1.0#n84)
/// - Current x86 `fd_set` layout:
///   [include/uapi/linux/posix_types.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/posix_types.h?h=v7.0#n22)
/// - Current i386 syscall table:
///   [arch/x86/entry/syscalls/syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n97)
pub unsafe fn select(
    nfds: Int,
    readfds: *mut FdSet,
    writefds: *mut FdSet,
    exceptfds: *mut FdSet,
    timeout: *mut Timeval,
) -> Int {
    let args = SelectArgs {
        nfds: nfds as isize as UnsignedLong,
        readfds: readfds.addr() as UnsignedLong,
        writefds: writefds.addr() as UnsignedLong,
        exceptfds: exceptfds.addr() as UnsignedLong,
        timeout: timeout.addr() as UnsignedLong,
    };

    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Select, (&raw const args).addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    use celer_system_linux_ctypes::{FdSet, Int, Timeval, UnsignedLong};

    use crate::arch::current::Sysno;
    use crate::sys::{
        SigHandler, close, exit, fork, getpid, kill, pipe, sig_handler,
        sig_handler_from_raw, signal, waitpid, write,
    };

    use super::{SelectArgs, select};

    const EINTR: Int = -(4 as Int);
    const EINVAL: Int = -(22 as Int);
    const SIGUSR1: Int = 10;

    static SELECT_SIGNAL_LOCK: Mutex<()> = Mutex::new(());

    extern "C" fn handle_sigalrm(_: Int) {}

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
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        if pid == 0 {
            thread::sleep(Duration::from_millis(100));
            let rc = kill(target, sig);
            if rc != 0 {
                exit(1);
            }
            exit(0);
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

    fn empty_fd_set() -> FdSet {
        FdSet { fds_bits: [0; 32] }
    }

    fn set_fd(set: &mut FdSet, fd: Int) {
        let bits_per_long = UnsignedLong::BITS as usize;
        let fd = fd as usize;
        let word = fd / bits_per_long;
        let bit = fd % bits_per_long;

        set.fds_bits[word] |= (1 as UnsignedLong) << bit;
    }

    fn is_fd_set(set: &FdSet, fd: Int) -> bool {
        let bits_per_long = UnsignedLong::BITS as usize;
        let fd = fd as usize;
        let word = fd / bits_per_long;
        let bit = fd % bits_per_long;

        (set.fds_bits[word] & ((1 as UnsignedLong) << bit)) != 0
    }

    #[test]
    fn test_select_sysno() {
        assert_eq!(Sysno::Select as isize, 82);
    }

    #[test]
    fn test_select_fd_set_layout() {
        assert_eq!(core::mem::size_of::<FdSet>(), 128);
        assert_eq!(core::mem::align_of::<FdSet>(), 4);
        assert_eq!(core::mem::offset_of!(FdSet, fds_bits), 0);
    }

    #[test]
    fn test_select_args_layout() {
        assert_eq!(core::mem::size_of::<SelectArgs>(), 20);
        assert_eq!(core::mem::align_of::<SelectArgs>(), 4);
        assert_eq!(core::mem::offset_of!(SelectArgs, nfds), 0);
        assert_eq!(core::mem::offset_of!(SelectArgs, readfds), 4);
        assert_eq!(core::mem::offset_of!(SelectArgs, writefds), 8);
        assert_eq!(core::mem::offset_of!(SelectArgs, exceptfds), 12);
        assert_eq!(core::mem::offset_of!(SelectArgs, timeout), 16);
    }

    #[test]
    fn test_select_pipe_read_end_ready() {
        let mut fds = [0 as Int; 2];
        // SAFETY: `fds` is writable for two `Int` values.
        let rc = unsafe { pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");

        let msg = [b'x'];
        let written = write(fds[1] as _, msg.as_ptr().cast(), msg.len());
        assert_eq!(written, 1, "write failed: {written}");

        let mut readfds = empty_fd_set();
        set_fd(&mut readfds, fds[0]);

        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        // SAFETY: `readfds` and `timeout` are valid writable syscall buffers.
        let ready = unsafe {
            select(
                fds[0] + 1,
                &raw mut readfds,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut timeout,
            )
        };

        assert_eq!(ready, 1, "select failed: {ready}");
        assert!(is_fd_set(&readfds, fds[0]));
        assert_eq!(close(fds[0]), 0);
        assert_eq!(close(fds[1]), 0);
    }

    #[test]
    fn test_select_zero_timeout_with_null_sets() {
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        // SAFETY: null descriptor sets are allowed and `timeout` is writable.
        let rc = unsafe {
            select(
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut timeout,
            )
        };

        assert_eq!(rc, 0, "select failed: {rc}");
        assert_eq!(timeout.tv_sec, 0);
        assert_eq!(timeout.tv_usec, 0);
    }

    #[test]
    fn test_select_negative_nfds_returns_einval() {
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        // SAFETY: null descriptor sets are allowed and `timeout` is writable.
        let rc = unsafe {
            select(
                -1,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut timeout,
            )
        };

        assert_eq!(rc, EINVAL, "expected EINVAL, got {rc}");
    }

    #[test]
    fn test_select_closed_fd_returns_ebadf() {
        let mut fds = [0 as Int; 2];
        // SAFETY: `fds` is writable for two `Int` values.
        let rc = unsafe { pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");

        assert_eq!(close(fds[0]), 0);

        let mut readfds = empty_fd_set();
        set_fd(&mut readfds, fds[0]);

        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        // SAFETY: `readfds` and `timeout` are valid writable syscall buffers.
        let rc = unsafe {
            select(
                fds[0] + 1,
                &raw mut readfds,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut timeout,
            )
        };

        assert_eq!(rc, -9, "expected EBADF, got {rc}");
        assert_eq!(close(fds[1]), 0);
    }

    #[test]
    fn test_select_bad_timeout_pointer_returns_efault() {
        // SAFETY: this intentionally passes an invalid timeout pointer to
        // verify the kernel's `EFAULT` path.
        let rc = unsafe {
            select(
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                usize::MAX as *mut Timeval,
            )
        };

        assert_eq!(rc, -14, "expected EFAULT, got {rc}");
    }

    #[test]
    fn test_select_negative_timeout_usec_returns_einval() {
        let mut timeout = Timeval {
            tv_sec: 0,
            tv_usec: -1,
        };

        // SAFETY: null descriptor sets are allowed and `timeout` is writable.
        let rc = unsafe {
            select(
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut timeout,
            )
        };

        assert_eq!(rc, EINVAL, "expected EINVAL, got {rc}");
    }

    #[test]
    fn test_select_interrupted_by_signal_returns_eintr() {
        let _guard = SELECT_SIGNAL_LOCK.lock().unwrap();

        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");
        if pid == 0 {
            let old = unsafe { signal(SIGUSR1, sig_handler(handle_sigalrm)) };
            assert!(old >= 0, "installing SIGUSR1 handler failed: {old}");
            let _restore = RestoreHandler {
                sig: SIGUSR1,
                old: sig_handler_from_raw(old),
            };

            let sender = spawn_signal_sender(getpid(), SIGUSR1);

            // SAFETY: null fd sets and null timeout are valid; this call
            // blocks until the signal sender interrupts it.
            let rc = unsafe {
                select(
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                )
            };

            let mut sender_status = 0;
            let waited = unsafe { waitpid(sender, &raw mut sender_status, 0) };
            assert_eq!(waited, sender, "waitpid failed: {waited}");
            assert_clean_exit(sender_status);
            assert_eq!(rc, EINTR, "expected EINTR, got {rc}");
            exit(0);
        }

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid, "waitpid failed: {waited}");
        assert_clean_exit(status);
    }
}
