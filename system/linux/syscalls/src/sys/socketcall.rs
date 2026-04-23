use celer_system_linux_ctypes::{Int, Long, UnsignedLong};

use crate::arch::current::{Sysno, syscall2};

/// Linux 1.0 `socketcall` subcall selector for `socket(2)`.
pub const SYS_SOCKET: Int = 1;
/// Linux 1.0 `socketcall` subcall selector for `bind(2)`.
pub const SYS_BIND: Int = 2;
/// Linux 1.0 `socketcall` subcall selector for `connect(2)`.
pub const SYS_CONNECT: Int = 3;
/// Linux 1.0 `socketcall` subcall selector for `listen(2)`.
pub const SYS_LISTEN: Int = 4;
/// Linux 1.0 `socketcall` subcall selector for `accept(2)`.
pub const SYS_ACCEPT: Int = 5;
/// Linux 1.0 `socketcall` subcall selector for `getsockname(2)`.
pub const SYS_GETSOCKNAME: Int = 6;
/// Linux 1.0 `socketcall` subcall selector for `getpeername(2)`.
pub const SYS_GETPEERNAME: Int = 7;
/// Linux 1.0 `socketcall` subcall selector for `socketpair(2)`.
pub const SYS_SOCKETPAIR: Int = 8;
/// Linux 1.0 `socketcall` subcall selector for `send(2)`.
pub const SYS_SEND: Int = 9;
/// Linux 1.0 `socketcall` subcall selector for `recv(2)`.
pub const SYS_RECV: Int = 10;
/// Linux 1.0 `socketcall` subcall selector for `sendto(2)`.
pub const SYS_SENDTO: Int = 11;
/// Linux 1.0 `socketcall` subcall selector for `recvfrom(2)`.
pub const SYS_RECVFROM: Int = 12;
/// Linux 1.0 `socketcall` subcall selector for `shutdown(2)`.
pub const SYS_SHUTDOWN: Int = 13;
/// Linux 1.0 `socketcall` subcall selector for `setsockopt(2)`.
pub const SYS_SETSOCKOPT: Int = 14;
/// Linux 1.0 `socketcall` subcall selector for `getsockopt(2)`.
pub const SYS_GETSOCKOPT: Int = 15;

/// Dispatch a Linux 1.0 socket operation through the historical
/// `socketcall(2)` multiplexor.
///
/// This wrapper targets the original Linux 1.0 x86 syscall slot 102 ABI.
/// The kernel entry takes a subcall selector plus a pointer to a packed
/// `unsigned long` argument vector whose exact length and pointee semantics
/// depend on the selected subcall.
///
/// # Safety
/// - `args` must point to a readable packed `UnsignedLong` array containing at
///   least the number of words required by `call`.
/// - Any userspace pointers encoded inside that packed array must satisfy the
///   selected socket operation's read/write validity requirements for the
///   duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.96a
/// - Behavior changes: Linux 1.0 supports selectors `1..=15`; current x86
///   kernels still expose syscall slot 102 as a compatibility `socketcall`
///   multiplexor, but they accept additional selectors beyond Linux 1.0
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - Subcall-dependent
///
/// # Behavior
/// - `call` selects one of the Linux 1.0 socket operations:
///   `socket`, `bind`, `connect`, `listen`, `accept`, `getsockname`,
///   `getpeername`, `socketpair`, `send`, `recv`, `sendto`, `recvfrom`,
///   `shutdown`, `setsockopt`, or `getsockopt`.
/// - The kernel reads arguments from `args` as packed `unsigned long` words.
/// - On success, returns the selected socket helper's raw return value without
///   any multiplexor-level translation.
/// - Linux 1.0 rejects unknown `call` values instead of attempting dispatch.
///
/// # Errors
/// - `EFAULT`: `args` is not readable for the packed word count required by
///   the selected Linux 1.0 subcall.
/// - `EINVAL`: `call` does not match any Linux 1.0 socket subcall selector.
///
/// All other reachable errors come from the selected socket helper rather than
/// from the `socketcall` entry path itself.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/socketcall.2.html)
/// - Stable:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/net/compat.c?h=v7.0#n423)
/// - LTS:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/net/compat.c?h=v6.18.18#n423)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/net/socket.c?h=1.0#n851)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n111)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.96a](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/net/socket.c?h=0.96a#n682)
/// - Linux 1.0 subcall selectors:
///   [include/linux/net.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/net.h?h=1.0#n27)
pub unsafe fn socketcall(call: Int, args: *const UnsignedLong) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Socketcall, call as isize, args.addr() as isize)
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, Long, UnsignedLong};

    use crate::arch::current::Sysno;
    use crate::sys::close;

    use super::{SYS_SOCKETPAIR, socketcall};

    const AF_UNIX: Int = 1;
    const SOCK_STREAM: Int = 1;

    #[test]
    fn test_socketcall_sysno() {
        assert_eq!(Sysno::Socketcall as isize, 102);
    }

    #[test]
    fn test_socketcall_invalid_selector_returns_einval() {
        let args = [0 as UnsignedLong; 6];

        // SAFETY: `args` is readable, but selector `0` is intentionally
        // invalid for the Linux 1.0 socketcall entry.
        let rc = unsafe { socketcall(0, args.as_ptr()) };

        assert_eq!(rc, -(22 as Long));
    }

    #[test]
    fn test_socketcall_null_args_returns_efault() {
        // SAFETY: selector `SYS_SOCKET` makes the kernel validate `args` for
        // three readable words; null is intentionally invalid here.
        let rc = unsafe { socketcall(super::SYS_SOCKET, core::ptr::null()) };

        assert_eq!(rc, -(14 as Long));
    }

    #[test]
    fn test_socketcall_socketpair_round_trip() {
        let mut fds = [-1 as Int; 2];
        let args = [
            AF_UNIX as UnsignedLong,
            SOCK_STREAM as UnsignedLong,
            0 as UnsignedLong,
            (&raw mut fds).addr() as UnsignedLong,
        ];

        // SAFETY: `args` is a readable four-word Linux 1.0 `socketpair`
        // argument block, and `fds` is writable for two `Int` values.
        let rc = unsafe { socketcall(SYS_SOCKETPAIR, args.as_ptr()) };

        assert_eq!(rc, 0, "socketcall(SYS_SOCKETPAIR) failed: {rc}");
        assert!(fds[0] >= 0, "invalid first socket fd: {}", fds[0]);
        assert!(fds[1] >= 0, "invalid second socket fd: {}", fds[1]);
        assert_ne!(fds[0], fds[1]);
        assert_eq!(close(fds[0]), 0);
        assert_eq!(close(fds[1]), 0);
    }
}
