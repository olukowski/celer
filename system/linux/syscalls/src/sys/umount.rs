use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::Sysno;
#[cfg(target_arch = "x86")]
use crate::arch::current::syscall1;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
use crate::arch::current::syscall2;

/// Unmount a filesystem or mount point.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.10 on i386; x86_64 and aarch64 expose the same
///   implementation through the two-argument `umount2` syscall slot
/// - Behavior changes: Linux 1.0 accepts either a mounted block-device path
///   or a mounted path and special-cases the root filesystem by remounting it
///   read-only; current kernels that still expose `oldumount` route it to the
///   flagless `ksys_umount(name, 0)` path
/// - Availability: present on supported x86, x86_64, and aarch64 Linux
///   kernels
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
/// - Stable path helper: [v7.0 path_umount](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v7.0#n2035)
/// - Stable shared helper: [v7.0 ksys_umount](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v7.0#n2050)
/// - Stable x86_64/aarch64 entry: [v7.0 umount](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v7.0#n2068)
/// - Stable x86 entry: [v7.0 oldumount](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v7.0#n2078)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n29)
/// - Stable x86_64 table (`umount2` slot): [v7.0 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n178)
/// - Stable aarch64 syscall numbers (`umount2` slot):
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n120)
/// - LTS path helper: [v6.18.18 path_umount](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2035)
/// - LTS shared helper: [v6.18.18 ksys_umount](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2050)
/// - LTS x86_64/aarch64 entry: [v6.18.18 umount](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2068)
/// - LTS x86 entry: [v6.18.18 oldumount](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2078)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n29)
/// - LTS x86_64 table (`umount2` slot): [v6.18.18 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v6.18.18#n178)
/// - LTS aarch64 syscall numbers (`umount2` slot):
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n120)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=1.0#n249)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=0.10#n166)
pub unsafe fn umount(name: *const Char) -> Int {
    // SAFETY: guaranteed by caller.
    #[cfg(target_arch = "x86")]
    unsafe {
        syscall1(Sysno::Umount, name.addr() as isize) as Int
    }

    // SAFETY: guaranteed by caller. The public wrapper is flagless, matching
    // the legacy `oldumount` behavior by passing flags as zero.
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    unsafe {
        syscall2(Sysno::Umount, name.addr() as isize, 0) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::Char;

    use crate::arch::current::Sysno;

    use super::umount;

    #[test]
    fn test_umount_syscall_number() {
        #[cfg(target_arch = "x86")]
        let expected = 22;
        #[cfg(target_arch = "aarch64")]
        let expected = 39;
        #[cfg(target_arch = "x86_64")]
        let expected = 166;

        assert_eq!(Sysno::Umount as isize, expected);
    }

    #[test]
    fn test_umount_invalid_name() {
        let path = c"/definitely/not/a/celer-mount-target";

        // SAFETY: `path` is NUL-terminated and valid for the syscall.
        let result = unsafe { umount(path.as_ptr().cast::<Char>()) };
        assert!(result < 0, "umount unexpectedly succeeded: {result}");
    }
}
