use celer_system_linux_ctypes::{Long, Statfs, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Get filesystem status information for an open file descriptor through the
/// original Linux 1.0 `fstatfs` ABI.
///
/// # Safety
/// - `buf` must point to writable memory for one [`Statfs`] value for the
///   duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 checked `buf` for writability before
///   validating `fd`, and its filesystem `statfs` callback returned `void`
///   instead of an errno value; current kernels use a different internal path
///   and a newer UAPI layout.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, writes filesystem statistics for the open file referenced by
///   `fd` into `buf`.
/// - Linux 1.0 validates `buf` before checking whether `fd` names an open
///   file descriptor.
/// - Linux 1.0 uses the historical [`Statfs`] layout, including `f_spare[6]`.
/// - Linux 1.0 calls the backing superblock's `statfs` hook only when that
///   hook exists.
/// - Linux 1.0's `statfs` hook returned `void`, so this syscall could not
///   propagate filesystem-specific errors from that callback.
///
/// # Errors
/// - `EFAULT`: `buf` is not writable for one [`Statfs`] value.
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `ENOENT`: `fd` refers to a file table entry whose inode pointer is null.
/// - `ENOSYS`: the backing superblock does not provide a `statfs` hook.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/statfs.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v6.19#n211)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/statfs.c?h=v6.18.18#n211)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n49)
///
/// # Historical References
/// - Linux 1.0 `struct statfs`: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
pub unsafe fn fstatfs(fd: UnsignedInt, buf: *mut Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Fstatfs, fd as isize, buf.addr() as isize) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs::File, os::fd::AsRawFd as _};

    use celer_system_linux_ctypes::{FsidT, Statfs, UnsignedInt};

    use crate::arch::current::Sysno;

    use super::fstatfs;

    #[test]
    fn test_fstatfs_sysno() {
        assert_eq!(Sysno::Fstatfs as isize, 100);
    }

    #[test]
    fn test_fstatfs_layout() {
        assert_eq!(core::mem::size_of::<FsidT>(), 8);
        assert_eq!(core::mem::align_of::<FsidT>(), 4);
        assert_eq!(core::mem::size_of::<Statfs>(), 64);
        assert_eq!(core::mem::align_of::<Statfs>(), 4);
        assert_eq!(core::mem::offset_of!(Statfs, f_type), 0);
        assert_eq!(core::mem::offset_of!(Statfs, f_bsize), 4);
        assert_eq!(core::mem::offset_of!(Statfs, f_blocks), 8);
        assert_eq!(core::mem::offset_of!(Statfs, f_bfree), 12);
        assert_eq!(core::mem::offset_of!(Statfs, f_bavail), 16);
        assert_eq!(core::mem::offset_of!(Statfs, f_files), 20);
        assert_eq!(core::mem::offset_of!(Statfs, f_ffree), 24);
        assert_eq!(core::mem::offset_of!(Statfs, f_fsid), 28);
        assert_eq!(core::mem::offset_of!(Statfs, f_namelen), 36);
        assert_eq!(core::mem::offset_of!(Statfs, f_spare), 40);
    }

    #[test]
    fn test_fstatfs_success() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        let mut buf = Statfs {
            f_type: -1,
            f_bsize: -1,
            f_blocks: -1,
            f_bfree: -1,
            f_bavail: -1,
            f_files: -1,
            f_ffree: -1,
            f_fsid: FsidT { val: [-1, -1] },
            f_namelen: -1,
            f_spare: [-1; 6],
        };

        // SAFETY: `buf` is writable for a full `Statfs`.
        let ret = unsafe { fstatfs(fd, &raw mut buf) };

        assert_eq!(ret, 0, "fstatfs failed for /: {ret}");
        assert_ne!(buf.f_type, -1, "expected kernel to fill f_type");
        assert!(buf.f_bsize > 0, "expected positive block size: {:?}", buf);
        assert!(
            buf.f_blocks >= 0,
            "expected nonnegative block count: {:?}",
            buf
        );
        assert!(
            buf.f_namelen >= 0,
            "expected nonnegative name length: {:?}",
            buf
        );
    }

    #[test]
    fn test_fstatfs_invalid_fd_returns_ebadf() {
        let mut buf = Statfs {
            f_type: 11,
            f_bsize: 22,
            f_blocks: 33,
            f_bfree: 44,
            f_bavail: 55,
            f_files: 66,
            f_ffree: 77,
            f_fsid: FsidT { val: [88, 99] },
            f_namelen: 111,
            f_spare: [123; 6],
        };

        // SAFETY: `buf` is writable for a full `Statfs`.
        let ret = unsafe { fstatfs(u32::MAX, &raw mut buf) };

        assert_eq!(ret, -9, "expected EBADF from invalid fd, got {ret}");
        assert_eq!(buf.f_type, 11, "buffer should remain untouched on EBADF");
    }

    #[test]
    fn test_fstatfs_null_buffer_returns_efault() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;

        // SAFETY: this test intentionally passes a null pointer to verify the
        // kernel's `EFAULT` path for a valid file descriptor.
        let ret = unsafe { fstatfs(fd, core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null buffer, got {ret}");
    }
}
