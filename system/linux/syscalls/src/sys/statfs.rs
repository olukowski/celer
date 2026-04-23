use celer_system_linux_ctypes::{Char, Long, Statfs};

use crate::arch::current::{Sysno, syscall2};

/// Return filesystem status for the filesystem containing `path` through the
/// historical Linux 1.0 `statfs` ABI.
///
/// Linux 1.0 exposes syscall number `99` as `sys_statfs(const char *path,
/// struct statfs *buf)`.
///
/// # Safety
/// - `buf` must be writable for one `Statfs` value for the duration of the
///   syscall, and the kernel write through `buf` must not violate Rust
///   aliasing or lifetime rules.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: modern kernels still expose `statfs` on x86, but later
///   kernels route through newer internal helpers and support more filesystems.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `buf` with the historical Linux `struct statfs` result
///   for the filesystem containing `path`.
/// - Linux 1.0 first validates that `buf` is writable for one `Statfs`, then
///   resolves `path` with the normal pathname lookup machinery.
/// - Linux 1.0 returns `-ENOSYS` when the resolved superblock does not provide
///   a `statfs` operation.
///
/// # Errors
/// - `EFAULT`: `buf` is not writable for one `Statfs`, or `path` points
///   outside the task address space.
/// - `ENAMETOOLONG`: `path` does not fit in the kernel pathname buffer used by
///   Linux 1.0 path lookup.
/// - `ENOENT`: `path` is empty, a path component does not exist, or lookup
///   otherwise fails with `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer.
/// - `ENOTDIR`: a non-directory component was used where pathname traversal
///   required a directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
/// - `ENOSYS`: the resolved filesystem does not implement the Linux 1.0
///   `statfs` superblock operation.
///
/// Linux 1.0 may also propagate filesystem-dependent lookup failures before it
/// reaches the superblock `statfs` method, including `ELOOP`, `EIO`, and some
/// NFS-specific lookup errors.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/statfs.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v6.19#n131)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/statfs.c?h=v6.18.18#n131)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n29)
///
/// # Historical References
/// - Linux 1.0 `struct statfs`: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
pub unsafe fn statfs(path: *const Char, buf: *mut Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Statfs, path.addr() as isize, buf.addr() as isize)
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, FsidT, Statfs};

    use crate::arch::current::Sysno;

    use super::statfs;

    fn zeroed_statfs() -> Statfs {
        Statfs {
            f_type: 0,
            f_bsize: 0,
            f_blocks: 0,
            f_bfree: 0,
            f_bavail: 0,
            f_files: 0,
            f_ffree: 0,
            f_fsid: FsidT { val: [0; 2] },
            f_namelen: 0,
            f_spare: [0; 6],
        }
    }

    #[repr(C)]
    struct StatfsWithCanary {
        statfs: Statfs,
        canary: [u8; 64],
    }

    #[test]
    fn test_statfs_layout() {
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
    fn test_statfs_syscall_number() {
        assert_eq!(Sysno::Statfs as isize, 99);
    }

    #[test]
    fn test_statfs_root_succeeds() {
        let path = b"/\0";
        let mut buf = zeroed_statfs();

        // SAFETY: `buf` is writable for one `Statfs` and no mutable aliases
        // are used during the syscall.
        let ret = unsafe { statfs(path.as_ptr().cast::<Char>(), &raw mut buf) };

        assert_eq!(ret, 0, "statfs failed for /: {ret}");
        assert!(buf.f_bsize > 0, "expected a positive block size");
    }

    #[test]
    fn test_statfs_missing_path() {
        let path = b"/definitely/not/present/celer_statfs\0";
        let mut buf = zeroed_statfs();

        // SAFETY: `buf` is writable for one `Statfs` and no mutable aliases
        // are used during the syscall.
        let ret = unsafe { statfs(path.as_ptr().cast::<Char>(), &raw mut buf) };

        assert_eq!(ret, -2, "expected ENOENT from statfs, got {ret}");
    }

    #[test]
    fn test_statfs_null_buffer_faults() {
        let path = b"/\0";
        // SAFETY: a null output pointer is permitted to test kernel `EFAULT`;
        // the wrapper remains unsafe because valid writable output buffers are
        // otherwise required.
        let ret = unsafe {
            statfs(path.as_ptr().cast::<Char>(), core::ptr::null_mut())
        };

        assert_eq!(ret, -14, "expected EFAULT from statfs, got {ret}");
    }

    #[test]
    fn test_statfs_uses_historical_copy_size() {
        let path = b"/\0";
        let mut buf = StatfsWithCanary {
            statfs: zeroed_statfs(),
            canary: [0xA5; 64],
        };

        // SAFETY: `buf.statfs` is writable for one `Statfs` and the canary is
        // only observed after the syscall returns.
        let ret = unsafe {
            statfs(path.as_ptr().cast::<Char>(), &raw mut buf.statfs)
        };

        assert_eq!(ret, 0, "statfs failed for /: {ret}");
        assert_eq!(buf.canary, [0xA5; 64], "statfs overwrote canary");
    }
}
