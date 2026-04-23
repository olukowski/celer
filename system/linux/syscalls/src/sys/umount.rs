use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Unmount a filesystem or mount point.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 accepts either a mounted block-device path
///   or a mounted path and special-cases the root filesystem by remounting it
///   read-only; current kernels that still expose `oldumount` route it to the
///   flagless `ksys_umount(name, 0)` path
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
/// - Current kernels require unmount permission, typically `CAP_SYS_ADMIN`.
///
/// # Behavior
/// - Linux 1.0 first resolves `name` through `namei()`, then retries with
///   `lnamei()` if the first lookup fails.
/// - If Linux 1.0 resolves `name` to a block-device inode, it unmounts the
///   corresponding mounted device.
/// - Otherwise Linux 1.0 requires the resolved inode to be exactly the
///   mounted root inode of its superblock.
/// - On Linux 1.0, unmounting the root filesystem remounts it read-only
///   instead of tearing it down fully.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to perform the unmount operation.
/// - `EINVAL`: Linux 1.0 resolves `name`, but it is neither a mounted block
///   device nor the mounted root inode of a superblock.
/// - `EACCES`: Linux 1.0 resolves `name` to a block-device inode whose device
///   is marked `nodev`.
/// - `ENXIO`: Linux 1.0 resolves `name` to a block-device inode whose major
///   number is outside the registered block-device table.
/// - `ENOENT`: Linux 1.0 cannot find `name` through either lookup path, or
///   the selected device has no mounted superblock to unmount.
/// - `EBUSY`: Linux 1.0 refuses to unmount because the target superblock is
///   still busy.
/// - Linux 1.0 also forwards additional pathname-resolution errors from
///   `namei()` and `lnamei()`, including common lookup failures such as
///   `ENOTDIR` and `EACCES`.
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Char;

    use super::umount;

    #[test]
    fn test_umount_invalid_name() {
        let path = c"/definitely/not/a/celer-mount-target";

        // SAFETY: `path` is NUL-terminated and valid for the syscall.
        let result = unsafe { umount(path.as_ptr().cast::<Char>()) };
        assert!(result < 0, "umount unexpectedly succeeded: {result}");
    }
}
