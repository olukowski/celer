use celer_system_linux_ctypes::{
    Long, NewStat, UnsignedInt, linux_1_0::NewStat as Linux10NewStat,
};

use crate::arch::{
    current::{Sysno, syscall2},
    linux_1_0::{Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2},
};

/// Get file status information for an open file descriptor through the
/// current i386 `newfstat` ABI.
///
/// Linux 1.0 exposes syscall number `108` as `fstat`, but wires that slot to
/// `sys_newfstat(unsigned int fd, struct new_stat *statbuf)`.
///
/// # Safety
/// - `statbuf` must point to writable memory for one [`NewStat`] value for
///   the duration of the syscall, and the kernel write through `statbuf` must
///   not violate Rust aliasing or lifetime rules.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: modern x86 kernels still expose this syscall number,
///   but later kernels route through `vfs_fstat()` and newer compat-copy
///   helpers with representability checks that Linux 1.0 did not perform.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `statbuf` with metadata for the open file referenced by
///   `fd`.
/// - Current kernels copy the i386 `struct stat` layout through
///   `cp_new_stat()`, including nanosecond timestamp fields.
///
/// # Errors
/// - `EFAULT`: `statbuf` is not writable for one [`NewStat`] value.
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `EOVERFLOW`: file metadata cannot be represented in the i386
///   `struct stat` layout.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fstat.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0#n546)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n550)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n168)
///
/// # Historical References
/// - Current i386 `struct stat`: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/include/uapi/asm/stat.h?h=v7.0#n10)
/// - Current copy-out: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0#n546)
/// - Linux 1.0 `struct new_stat`, preserved as [`celer_system_linux_ctypes::linux_1_0::NewStat`]: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
/// - Linux 1.0 syscall number: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n117)
pub unsafe fn newfstat(fd: UnsignedInt, statbuf: *mut NewStat) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(Sysno::Newfstat, fd as isize, statbuf.addr() as isize) as Long
    }
}

/// Get file status information through the Linux 1.0 `sys_newfstat` ABI.
///
/// This wrapper uses syscall slot `108` with the Linux 1.0
/// [`Linux10NewStat`] layout. Current kernels use the same slot with the
/// current i386 [`NewStat`] layout, exposed by [`newfstat`].
///
/// # Safety
/// - `statbuf` must point to writable memory for one [`Linux10NewStat`] value
///   for the duration of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n168)
/// - Linux 1.0 `struct new_stat`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
pub unsafe fn newfstat_1_0(
    fd: UnsignedInt,
    statbuf: *mut Linux10NewStat,
) -> Long {
    // SAFETY: guaranteed by caller.
    unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Newfstat,
            fd as isize,
            statbuf.addr() as isize,
        ) as Long
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::File, mem::size_of, os::fd::AsRawFd as _,
        os::unix::fs::MetadataExt as _,
    };

    use celer_system_linux_ctypes::{
        NewStat, UnsignedInt, linux_1_0::NewStat as Linux10NewStat,
    };

    use crate::arch::{current::Sysno, linux_1_0::Sysno as Linux10Sysno};

    use super::{newfstat, newfstat_1_0};

    fn zeroed_newstat() -> NewStat {
        NewStat {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused4: 0,
            __unused5: 0,
        }
    }

    fn zeroed_linux_1_0_newstat() -> Linux10NewStat {
        Linux10NewStat {
            st_dev: 0,
            __pad1: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad2: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            __unused1: 0,
            st_mtime: 0,
            __unused2: 0,
            st_ctime: 0,
            __unused3: 0,
            __unused4: 0,
            __unused5: 0,
        }
    }

    #[repr(C)]
    struct NewStatWithCanary {
        stat: NewStat,
        canary: [u8; 64],
    }

    #[test]
    fn test_newfstat_layout() {
        assert_eq!(size_of::<NewStat>(), 64);
        assert_eq!(core::mem::align_of::<NewStat>(), 4);
        assert_eq!(core::mem::offset_of!(NewStat, st_dev), 0);
        assert_eq!(core::mem::offset_of!(NewStat, st_ino), 4);
        assert_eq!(core::mem::offset_of!(NewStat, st_mode), 8);
        assert_eq!(core::mem::offset_of!(NewStat, st_nlink), 10);
        assert_eq!(core::mem::offset_of!(NewStat, st_uid), 12);
        assert_eq!(core::mem::offset_of!(NewStat, st_gid), 14);
        assert_eq!(core::mem::offset_of!(NewStat, st_rdev), 16);
        assert_eq!(core::mem::offset_of!(NewStat, st_size), 20);
        assert_eq!(core::mem::offset_of!(NewStat, st_blksize), 24);
        assert_eq!(core::mem::offset_of!(NewStat, st_blocks), 28);
        assert_eq!(core::mem::offset_of!(NewStat, st_atime), 32);
        assert_eq!(core::mem::offset_of!(NewStat, st_atime_nsec), 36);
        assert_eq!(core::mem::offset_of!(NewStat, st_mtime), 40);
        assert_eq!(core::mem::offset_of!(NewStat, st_mtime_nsec), 44);
        assert_eq!(core::mem::offset_of!(NewStat, st_ctime), 48);
        assert_eq!(core::mem::offset_of!(NewStat, st_ctime_nsec), 52);
        assert_eq!(core::mem::offset_of!(NewStat, __unused4), 56);
        assert_eq!(core::mem::offset_of!(NewStat, __unused5), 60);
    }

    #[test]
    fn test_newfstat_syscall_number() {
        assert_eq!(Sysno::Newfstat as isize, 108);
        assert_eq!(Linux10Sysno::Newfstat as isize, 108);
    }

    #[test]
    fn test_newfstat_success() {
        let file = File::open("/").unwrap();
        let metadata = file.metadata().unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        let mut statbuf = zeroed_newstat();

        // SAFETY: `statbuf` is writable for one `NewStat` and no mutable
        // aliases are used during the syscall.
        let ret = unsafe { newfstat(fd, &raw mut statbuf) };

        assert_eq!(ret, 0, "newfstat failed for /: {ret}");
        assert_eq!(statbuf.st_ino as u64, metadata.ino());
        assert_eq!(statbuf.st_mode as u32, metadata.mode());
        assert_eq!(statbuf.st_nlink as u64, metadata.nlink());
        assert_eq!(statbuf.st_size as u64, metadata.size());
    }

    #[test]
    fn test_linux_1_0_newfstat_wrapper_success() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        let mut statbuf = zeroed_linux_1_0_newstat();

        // SAFETY: `statbuf` is writable for one Linux 1.0 `NewStat`.
        let ret = unsafe { newfstat_1_0(fd, &raw mut statbuf) };

        assert_eq!(ret, 0, "linux_1_0::newfstat failed for /: {ret}");
    }

    #[test]
    fn test_newfstat_invalid_fd_returns_ebadf() {
        let mut statbuf = zeroed_newstat();

        // SAFETY: `statbuf` is writable for one `NewStat`.
        let ret = unsafe { newfstat(u32::MAX, &raw mut statbuf) };

        assert_eq!(ret, -9, "expected EBADF from invalid fd, got {ret}");
    }

    #[test]
    fn test_newfstat_null_buffer_returns_efault() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;

        // SAFETY: this test intentionally passes a null pointer to verify the
        // kernel's `EFAULT` path for a valid file descriptor.
        let ret = unsafe { newfstat(fd, core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null buffer, got {ret}");
    }

    #[test]
    fn test_newfstat_uses_current_stat_copy_size() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as UnsignedInt;
        let mut statbuf = NewStatWithCanary {
            stat: zeroed_newstat(),
            canary: [0xA5; 64],
        };

        // SAFETY: `statbuf.stat` is writable for one `NewStat` and the canary
        // is only observed after the syscall returns.
        let ret = unsafe { newfstat(fd, &raw mut statbuf.stat) };

        assert_eq!(ret, 0, "newfstat failed for /: {ret}");
        assert_eq!(statbuf.canary, [0xA5; 64], "newfstat overwrote canary");
    }
}
