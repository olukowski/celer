use celer_system_linux_ctypes::{Long, UnsignedLong};

use crate::arch::current::{Sysno, syscall4};

/// Control or inspect another process with `ptrace`.
///
/// This is the Linux `ptrace` syscall entry point.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: common Linux syscall
///
/// # Required Privileges
/// - Depends on the specific `request`.
///
/// # Behavior
/// - Dispatches the request to the kernel's ptrace implementation.
/// - Returns `0` or a positive result on success, depending on the request.
/// - Returns a negative errno value on failure.
///
/// # Errors
/// - `ESRCH`: the target task does not exist.
/// - `EPERM`: the caller lacks permission for the request.
/// - `EIO`: the request is not valid for the target or request-specific access failed.
/// - `EFAULT`: a user pointer argument does not point to accessible memory.
///
/// # Safety
/// - The caller must ensure `request`, `pid`, `addr`, and `data` are valid for
///   the chosen ptrace request.
/// - Any pointer encoded in `addr` or `data` must be valid for the duration of
///   the syscall and obey the specific request ABI.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/ptrace.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/ptrace.c?h=v6.19#n1387)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/ptrace.c?h=v6.18.18#n1387)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n26)
pub unsafe fn ptrace(
    request: Long,
    pid: Long,
    addr: UnsignedLong,
    data: UnsignedLong,
) -> Long {
    // SAFETY: the caller is responsible for request-specific argument validity.
    unsafe {
        syscall4(
            Sysno::Ptrace,
            request as isize,
            pid as isize,
            addr as isize,
            data as isize,
        ) as Long
    }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Long;

    use super::ptrace;

    const INVALID_REQUEST: Long = 9999;

    #[test]
    fn test_ptrace_executes() {
        // SAFETY: this intentionally uses an invalid request so the kernel
        // rejects it without mutating any ptrace state in the test process.
        let ret = unsafe { ptrace(INVALID_REQUEST, 0, 0, 0) };
        assert!(ret < 0);
    }
}
