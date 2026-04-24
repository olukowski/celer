use celer_system_linux_ctypes::{Char, Long, UnsignedLong, Void};

use crate::arch::current::{Sysno, syscall5};

/// Mount or remount a filesystem.
///
/// # Safety
/// - `target` must be valid to read a NUL-terminated string for the duration
///   of the syscall.
/// - Any non-null `source` and `filesystemtype` pointers must be valid to
///   read NUL-terminated strings for the duration of the syscall.
/// - Any non-null `data` pointer must satisfy the memory contract for the
///   selected filesystem and mount flags.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 1.0 only honors `mountflags` and `data` when the
///   high 16 bits of `mountflags` equal `0xC0ED`, and it enters a special
///   remount path only when that compatibility magic is present together with
///   `MS_REMOUNT`; current kernels still expose the five-argument `mount`
///   syscall, but they always copy and interpret the supplied flags and mount
///   data directly
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
/// - Current kernels require mount permission, typically `CAP_SYS_ADMIN`.
///
/// # Behavior
/// - Linux 1.0 copies `filesystemtype` into a temporary kernel page and
///   resolves it through the registered filesystem-type table.
/// - Linux 1.0 treats a null `filesystemtype` as a request for the first
///   registered filesystem type.
/// - On Linux 1.0, if `mountflags` does not carry the `MS_MGC_VAL`
///   compatibility magic in its high 16 bits, the syscall ignores both
///   `mountflags` and `data` and behaves like the older pre-flag ABI.
/// - On Linux 1.0, if `mountflags` carries `MS_MGC_VAL | MS_REMOUNT`, the
///   syscall remounts the filesystem already mounted on `target`; in that
///   path, `source` and `filesystemtype` are ignored after the mount-data
///   copy.
/// - If the selected filesystem type requires a device, Linux 1.0 resolves
///   `source` and requires it to be a block-device inode with a valid device
///   number.
/// - If the selected filesystem type does not require a device, Linux 1.0
///   allocates an unnamed synthetic device number before mounting it on
///   `target`.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to perform the mount, or Linux 1.0
///   finds that `target` is not a directory in the non-remount path.
/// - `EFAULT`: Linux 1.0 cannot copy `filesystemtype` or `data` because the
///   supplied pointer lies outside the calling task's mapped address ranges.
/// - `ENOMEM`: Linux 1.0 cannot allocate the temporary page used to copy
///   `filesystemtype` or `data`.
/// - `ENODEV`: Linux 1.0 cannot resolve `filesystemtype` to a registered
///   filesystem type.
/// - `ENOTBLK`: Linux 1.0 resolves `source`, but the selected filesystem type
///   requires a block device and the resolved inode is not block-backed.
/// - `EACCES`: Linux 1.0 resolves `source` as a block device whose inode is
///   marked `nodev`.
/// - `ENXIO`: Linux 1.0 resolves `source` as a block device whose major
///   number is outside the registered block-device table.
/// - `EMFILE`: Linux 1.0 needs an unnamed synthetic device for a device-less
///   filesystem type, but none are free.
/// - `EBUSY`: Linux 1.0 finds `target` already mounted or busy, rejects the
///   candidate device as already mounted or otherwise not mountable, cannot
///   obtain a usable superblock, or the remount path refuses a read-only
///   transition with active writable files.
/// - `EINVAL`: Linux 1.0 remounts `target`, but `target` is not the mounted
///   root inode of its superblock.
/// - Linux 1.0 also forwards pathname-resolution and block-driver open errors
///   from resolving `source` or `target`, including common lookup failures
///   such as `ENOENT`, `ENOTDIR`, and `EACCES`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/mount.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v6.19#n4201)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n4216)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=1.0#n427)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=0.10#n199)
pub unsafe fn mount(
    source: *const Char,
    target: *const Char,
    filesystemtype: *const Char,
    mountflags: UnsignedLong,
    data: *const Void,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall5(
            Sysno::Mount,
            source.addr() as isize,
            target.addr() as isize,
            filesystemtype.addr() as isize,
            mountflags as isize,
            data.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, UnsignedLong};

    use super::mount;

    #[test]
    fn test_mount_invalid_parameters() {
        let source = c"/definitely/not/a/celer-mount-source";
        let target = c"/definitely/not/a/celer-mount-target";
        let fstype = c"definitely-not-a-celer-fs";

        // SAFETY: all non-null string pointers are NUL-terminated and remain
        // valid for the duration of the syscall.
        let ret = unsafe {
            mount(
                source.as_ptr().cast::<Char>(),
                target.as_ptr().cast::<Char>(),
                fstype.as_ptr().cast::<Char>(),
                0 as UnsignedLong,
                core::ptr::null(),
            )
        };

        assert!(ret < 0, "mount unexpectedly succeeded: {}", ret);
    }
}
