use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall2};

/// Set the domain name for the current UTS namespace.
///
/// # Safety
/// - `name` must be valid to read `len` bytes for the duration of the
///   syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 required the superuser and copied bytes
///   until the first NUL byte or `len`; current kernels require
///   `CAP_SYS_ADMIN` in the owning user namespace of the current UTS
///   namespace and copy exactly `len` bytes, including embedded NUL bytes.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser.
/// - Current kernels: the caller must have `CAP_SYS_ADMIN` in the user
///   namespace that owns the current UTS namespace.
///
/// # Behavior
/// - `len` is the signed kernel byte count.
/// - On current kernels, `len` must be in the range `0..=64`.
/// - The wrapper passes exactly `len` bytes to the kernel; no trailing NUL
///   byte is required.
/// - Current kernels copy exactly `len` bytes and zero-fill the remaining
///   domainname buffer.
/// - Linux 1.0 copied byte-by-byte, returned immediately after copying a NUL
///   byte, and only wrote a trailing NUL at index `len` when no earlier NUL
///   byte was seen. Linux 1.0 rejected `len > 64` but did not contain an
///   entry-path check for negative `len`.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to change the domain name.
/// - `EINVAL`: on current kernels, `len` is negative or exceeds the kernel's
///   64-byte domain-name limit.
/// - `EINVAL`: on Linux 1.0, `len` exceeds the kernel's 64-byte
///   domain-name limit.
/// - `EFAULT`: on current kernels, `name` is not readable for `len` bytes.
///   Linux 1.0's syscall body does not contain a verified `EFAULT` return.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setdomainname.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1473)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1473)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n669)
pub unsafe fn setdomainname(name: *const Char, len: Int) -> Int {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Setdomainname, name.addr() as isize, len as isize)
    }) as Int
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, Int};

    use crate::arch::current::Sysno;

    use super::setdomainname;

    #[test]
    fn test_setdomainname_sysno() {
        assert_eq!(Sysno::Setdomainname as isize, 121);
    }

    #[test]
    fn test_setdomainname_name_too_long_or_permission_denied() {
        let name = [b'a'; 65];
        let ret =
            // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
            unsafe { setdomainname(name.as_ptr().cast::<Char>(), name.len() as Int) };

        // Unprivileged runs stop at `EPERM`; a namespace-isolated privileged
        // test would be needed to force the `EINVAL` length check.
        assert!(
            ret == -1 || ret == -22,
            "expected EPERM or EINVAL from setdomainname(65 bytes), got {ret}",
        );
    }

    #[test]
    fn test_setdomainname_negative_length_is_rejected_or_permission_denied() {
        let name = c"";

        // SAFETY: `name` is valid; current kernels reject `len < 0` before
        // reading any bytes.
        let ret = unsafe { setdomainname(name.as_ptr().cast::<Char>(), -1) };

        // Current kernels reject negative lengths with `EINVAL`, but the
        // privilege check runs first for unprivileged callers.
        assert!(
            ret == -1 || ret == -22,
            "expected EPERM or EINVAL from setdomainname(len = -1), got {ret}",
        );
    }
}
