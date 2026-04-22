use celer_system_linux_ctypes::{Char, Int, SizeT};

use crate::arch::current::{Sysno, syscall2};

/// Set the hostname for the current UTS namespace.
///
/// # Safety
/// - `name` must be readable for `len` bytes (see [`core::ptr::read`]).
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 required the superuser and copied bytes until
///   the first NUL byte or `len`; current kernels require `CAP_SYS_ADMIN` in
///   the owning user namespace of the current UTS namespace and copy exactly
///   `len` bytes, including embedded NUL bytes.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser.
/// - Current kernels: the caller must have `CAP_SYS_ADMIN` in the user
///   namespace that owns the current UTS namespace.
///
/// # Behavior
/// - `len` must not exceed 64 bytes.
/// - The wrapper passes exactly `len` bytes to the kernel; no trailing NUL
///   byte is required.
/// - Current kernels copy exactly `len` bytes and zero-fill the
///   remaining hostname buffer.
/// - Linux 1.0 copied byte-by-byte, returned immediately after copying a NUL
///   byte, and only appended a trailing NUL at index `len` when no
///   earlier NUL byte was seen.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to change the hostname.
/// - `EINVAL`: `len` exceeds the kernel's 64-byte hostname limit.
/// - `EFAULT`: `name` is not readable for `len` bytes.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/sethostname.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1419)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1419)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n649)
pub unsafe fn sethostname(name: *const Char, len: SizeT) -> Int {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Sethostname, name.addr() as isize, len as isize)
    }) as Int
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::{Char, SizeT};

    use super::sethostname;

    #[test]
    fn test_sethostname_name_too_long_or_permission_denied() {
        let name = [b'a'; 65];
        // SAFETY: `name.as_ptr()` is readable for `name.len()` bytes.
        let ret = unsafe {
            sethostname(name.as_ptr().cast::<Char>(), name.len() as SizeT)
        };

        // Unprivileged runs stop at `EPERM`; a namespace-isolated privileged
        // test would be needed to force the `EINVAL` length check.
        assert!(
            ret == -1 || ret == -22,
            "expected EPERM or EINVAL from sethostname(65 bytes), got {ret}",
        );
    }
}
