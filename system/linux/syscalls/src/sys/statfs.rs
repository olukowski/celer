#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Statfs as Linux10Statfs;
use celer_system_linux_ctypes::{Char, Long, Statfs};

use crate::arch::current::{Sysno, syscall2};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2,
};

/// Return filesystem status for the filesystem containing `path` through the
/// current i386 `statfs` ABI.
///
/// Current x86 exposes syscall number `99` as `statfs(const char __user *path,
/// struct statfs __user *buf)`.
///
/// # Safety
/// - `path` must point to a readable NUL-terminated string for the duration
///   of the syscall.
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
/// - On success, fills `buf` with the current i386 Linux `struct statfs`
///   result for the filesystem containing `path`.
/// - The current i386 layout uses unsigned 32-bit statfs words, `f_frsize`,
///   `f_flags`, and four spare words.
///
/// # Errors
/// - `EFAULT`: `buf` is not writable for one `Statfs`, or `path` points
///   outside the task address space.
/// - `ENAMETOOLONG`: `path` does not fit in the kernel pathname buffer used by
///   path lookup.
/// - `ENOENT`: `path` is empty, a path component does not exist, or lookup
///   otherwise fails with `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer.
/// - `ENOTDIR`: a non-directory component was used where pathname traversal
///   required a directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
/// - `ENOSYS`: the resolved filesystem does not implement a `statfs`
///   superblock operation.
/// - `EOVERFLOW`: filesystem statistics cannot be represented in the i386
///   `struct statfs` layout.
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
/// - Current `struct statfs`: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/statfs.h?h=v7.0#n23)
/// - Current copy-out: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v7.0#n125)
/// - Linux 1.0 `struct statfs`, preserved as [`celer_system_linux_ctypes::linux_1_0::Statfs`]: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
pub unsafe fn statfs(path: *const Char, buf: *mut Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(Sysno::Statfs, path.addr() as isize, buf.addr() as isize)
    }) as Long
}

/// Return filesystem status through the Linux 1.0 `sys_statfs` ABI.
///
/// This wrapper uses syscall slot `99` with the Linux 1.0 [`Linux10Statfs`]
/// layout. Current kernels use the same slot with the current i386 [`Statfs`]
/// layout, exposed by [`statfs`].
///
/// # Safety
/// - `path` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `buf` must point to writable memory for one [`Linux10Statfs`] value for
///   the duration of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n29)
/// - Linux 1.0 `struct statfs`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
#[cfg(target_arch = "x86")]
pub unsafe fn statfs_1_0(path: *const Char, buf: *mut Linux10Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Statfs,
            path.addr() as isize,
            buf.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{
        Char, FsidT, Statfs, linux_1_0, linux_1_0::Statfs as Linux10Statfs,
    };

    use crate::arch::{current::Sysno, linux_1_0::Sysno as Linux10Sysno};

    use super::{statfs, statfs_1_0};

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
            f_frsize: 0,
            f_flags: 0,
            f_spare: [0; 4],
        }
    }

    fn zeroed_linux_1_0_statfs() -> Linux10Statfs {
        Linux10Statfs {
            f_type: 0,
            f_bsize: 0,
            f_blocks: 0,
            f_bfree: 0,
            f_bavail: 0,
            f_files: 0,
            f_ffree: 0,
            f_fsid: linux_1_0::FsidT { val: [0; 2] },
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
        assert_eq!(core::mem::offset_of!(Statfs, f_frsize), 40);
        assert_eq!(core::mem::offset_of!(Statfs, f_flags), 44);
        assert_eq!(core::mem::offset_of!(Statfs, f_spare), 48);
    }

    #[test]
    fn test_statfs_syscall_number() {
        assert_eq!(Sysno::Statfs as isize, 99);
        assert_eq!(Linux10Sysno::Statfs as isize, 99);
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
    fn test_linux_1_0_statfs_wrapper_root_succeeds() {
        let path = b"/\0";
        let mut buf = zeroed_linux_1_0_statfs();

        // SAFETY: `path` is NUL-terminated and `buf` is writable for one
        // Linux 1.0 `Statfs`.
        let ret =
            unsafe { statfs_1_0(path.as_ptr().cast::<Char>(), &raw mut buf) };

        assert_eq!(ret, 0, "linux_1_0::statfs failed for /: {ret}");
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
    fn test_statfs_uses_current_statfs_copy_size() {
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
