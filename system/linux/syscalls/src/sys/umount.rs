use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Unmount a filesystem or mount point.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - The kernel rejects some requests unless the caller is allowed to mount
///   and unmount filesystems.
///
/// # Behavior
/// - Resolves `name` to a mount point or mounted block device.
/// - Unmounts the resolved target when the kernel accepts the request.
/// - Returns a negative errno value on failure.
///
/// # Errors
/// - `EINVAL`: The target is not a mounted path.
/// - `EPERM`: The caller is not permitted to perform the unmount operation.
///
/// # Safety
/// - `name` must point to a readable NUL-terminated string.
/// - Any irreversible side effects of unmounting are intended.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/umount.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v6.19#n2046)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2046)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=1.0#n249)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=0.10#n166)
pub unsafe fn umount(name: *const Char) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Umount, name.addr() as isize) as Int }
}

#[cfg(test)]
mod tests {
    use super::umount;

    #[test]
    fn test_umount_invalid_name() {
        // SAFETY: the kernel rejects this invalid pointer; the test only checks
        // that the wrapper reaches the syscall path.
        let result = unsafe { umount(core::ptr::null()) };
        assert!(result < 0, "umount unexpectedly succeeded: {result}");
    }
}
