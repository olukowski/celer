#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Statfs as Linux10Statfs;
use celer_system_linux_ctypes::{Long, Statfs, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2,
};

/// Get filesystem status information for an open file descriptor.
///
/// This wrapper uses the native `fstatfs(unsigned int fd, struct statfs __user
/// *buf)` syscall for the target architecture. On x86, the syscall number
/// differs by ABI:
/// - x86_64 uses syscall slot `138` and copies out the native 64-bit
///   `struct statfs`.
/// - i386 uses syscall slot `100` and copies out the native 32-bit
///   `struct statfs`; x86_64 compat tasks instead reach `compat_sys_fstatfs`
///   and receive `struct compat_statfs`.
///
/// # Safety
/// - `buf` must point to writable memory for one [`Statfs`] value for the
///   duration of the syscall.
/// - `buf` must not alias live Rust references or other memory that would
///   violate Rust's aliasing rules while the kernel may write through that
///   pointer.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes:
///   - Linux 1.0 i386 checked `buf` for writability before validating `fd`,
///     and its filesystem `statfs` callback returned `void` instead of an
///     errno value.
///   - Current kernels route native `fstatfs` through `fd_statfs()` and
///     `do_statfs_native()`.
///   - Current x86_64 native `fstatfs` uses the generic 64-bit
///     `struct statfs` layout from `asm-generic/statfs.h`.
///   - Current i386 native `fstatfs` and x86_64 compat `compat_sys_fstatfs`
///     use 32-bit statfs words and can fail with `EOVERFLOW` when `kstatfs`
///     values do not fit.
/// - Availability: present on supported x86 and x86_64 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, writes filesystem statistics for the open file referenced by
///   `fd` into `buf`.
/// - On x86_64, `do_statfs_native()` copies the native 64-bit
///   [`Statfs`] layout.
/// - On i386 native `fstatfs`, the current layout uses 32-bit statfs words,
///   plus `f_frsize`, `f_flags`, and `f_spare[4]`.
/// - On x86_64 compat `compat_sys_fstatfs`, the kernel converts `kstatfs`
///   into `struct compat_statfs`, which also uses 32-bit statfs words and the
///   same representability checks as current i386 native `fstatfs`.
///
/// # Errors
/// - `EFAULT`: `buf` is not writable for one [`Statfs`] value.
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `ENOSYS`: the backing superblock does not provide a `statfs` hook.
/// - `EOVERFLOW`: filesystem statistics cannot be represented in the current
///   32-bit native or compat `struct statfs` layout. This error does not
///   arise from the native x86_64 `do_statfs_native()` copy path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/statfs.2.html)
/// - Current native implementation:
///   [v7.0 fs/statfs.c](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v7.0#n212)
/// - Current native copy-out helper:
///   [v7.0 fs/statfs.c](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v7.0#n125)
/// - Current compat implementation:
///   [v7.0 fs/statfs.c](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v7.0#n313)
/// - Current x86 syscall tables:
///   [v7.0 arch/x86/entry/syscalls/syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n115),
///   [v7.0 arch/x86/entry/syscalls/syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n150)
/// - Linux 1.0 i386 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n49)
///
/// # Historical References
/// - Current native `struct statfs`:
///   [v7.0 include/uapi/asm-generic/statfs.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/statfs.h?h=v7.0#n23)
/// - Current x86 compat packing override:
///   [v7.0 arch/x86/include/uapi/asm/statfs.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/include/uapi/asm/statfs.h?h=v7.0#n5)
/// - Linux 1.0 `struct statfs`, preserved as `celer_system_linux_ctypes::linux_1_0::Statfs`: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
pub unsafe fn fstatfs(fd: UnsignedInt, buf: *mut Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Fstatfs, fd as isize, buf.addr() as isize) as Long
    }
}

/// Get filesystem status through the Linux 1.0 `sys_fstatfs` ABI.
///
/// This wrapper uses syscall slot `100` with the Linux 1.0 [`Linux10Statfs`]
/// layout. Current kernels use the same slot with the current i386 [`Statfs`]
/// layout, exposed by [`fstatfs`].
///
/// # Safety
/// - `buf` must point to writable memory for one [`Linux10Statfs`] value for
///   the duration of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n49)
/// - Linux 1.0 `struct statfs`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/vfs.h?h=1.0#n8)
#[cfg(target_arch = "x86")]
pub unsafe fn fstatfs_1_0(fd: UnsignedInt, buf: *mut Linux10Statfs) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Fstatfs,
            fd as isize,
            buf.addr() as isize,
        ) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs::File, os::fd::AsRawFd as _};

    use celer_system_linux_ctypes::{FsidT, Statfs, UnsignedInt};
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::{
        linux_1_0, linux_1_0::Statfs as Linux10Statfs,
    };

    use crate::arch::current::Sysno;
    #[cfg(target_arch = "x86")]
    use crate::arch::linux_1_0::Sysno as Linux10Sysno;

    use super::fstatfs;
    #[cfg(target_arch = "x86")]
    use super::fstatfs_1_0;

    #[cfg(target_arch = "x86")]
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

    #[test]
    fn test_fstatfs_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Fstatfs as isize, 100);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Fstatfs as isize, 44);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(Sysno::Fstatfs as isize, 138);
        #[cfg(target_arch = "x86")]
        assert_eq!(Linux10Sysno::Fstatfs as isize, 100);
    }

    #[test]
    fn test_fstatfs_layout() {
        assert_eq!(core::mem::size_of::<FsidT>(), 8);
        assert_eq!(core::mem::align_of::<FsidT>(), 4);
        assert_eq!(core::mem::offset_of!(Statfs, f_type), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(core::mem::size_of::<Statfs>(), 64);
            assert_eq!(core::mem::align_of::<Statfs>(), 4);
            assert_eq!(core::mem::offset_of!(Statfs, f_bsize), 4);
            assert_eq!(core::mem::offset_of!(Statfs, f_blocks), 8);
            assert_eq!(core::mem::offset_of!(Statfs, f_spare), 48);
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        {
            assert_eq!(core::mem::size_of::<Statfs>(), 120);
            assert_eq!(core::mem::align_of::<Statfs>(), 8);
            assert_eq!(core::mem::offset_of!(Statfs, f_bsize), 8);
            assert_eq!(core::mem::offset_of!(Statfs, f_blocks), 16);
            assert_eq!(core::mem::offset_of!(Statfs, f_spare), 88);
        }
    }

    #[test]
    fn test_fstatfs_success() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        #[cfg(target_arch = "x86")]
        let sentinel = u32::MAX;
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        let sentinel = i64::from(u32::MAX);
        let mut buf = Statfs {
            f_type: sentinel,
            f_bsize: sentinel,
            f_blocks: sentinel,
            f_bfree: sentinel,
            f_bavail: sentinel,
            f_files: sentinel,
            f_ffree: sentinel,
            f_fsid: FsidT { val: [-1, -1] },
            f_namelen: sentinel,
            f_frsize: sentinel,
            f_flags: sentinel,
            f_spare: [sentinel; 4],
        };

        // SAFETY: `buf` is writable for a full `Statfs`.
        let ret = unsafe { fstatfs(fd, &raw mut buf) };

        assert_eq!(ret, 0, "fstatfs failed for /: {ret}");
        assert_ne!(buf.f_type, sentinel, "expected kernel to fill f_type");
        assert!(buf.f_bsize > 0, "expected positive block size: {:?}", buf);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_fstatfs_wrapper_success() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        let mut buf = zeroed_linux_1_0_statfs();

        // SAFETY: `buf` is writable for one Linux 1.0 `Statfs`.
        let ret = unsafe { fstatfs_1_0(fd, &raw mut buf) };

        assert_eq!(ret, 0, "linux_1_0::fstatfs failed for /: {ret}");
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
            f_frsize: 222,
            f_flags: 333,
            f_spare: [123; 4],
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
