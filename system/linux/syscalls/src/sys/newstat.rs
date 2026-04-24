use celer_system_linux_ctypes::{
    Char, Long, NewStat, linux_1_0::NewStat as Linux10NewStat,
};

use crate::arch::{
    current::{Sysno, syscall2},
    linux_1_0::{Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2},
};

/// Get file status information for the path named by `filename` through the
/// current i386 `newstat` ABI.
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `statbuf` must point to writable memory large enough for a `NewStat`
///   value, and no other pointer or reference may alias that output for
///   mutable access for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `statbuf` with metadata for the file resolved from
///   `filename`.
/// - The final pathname component is followed if it is a symlink.
/// - The result uses the current i386 `struct stat` layout copied by
///   `cp_new_stat()`, including nanosecond timestamp fields.
///
/// # Errors
/// - `EFAULT`: `statbuf` is not writable for one `NewStat`, or `filename`
///   points outside the task address space.
/// - `ENAMETOOLONG`: `filename` does not fit in the single-page pathname
///   buffer used by `getname()`.
/// - `ENOENT`: `filename` is empty, names a missing path component, or
///   pathname lookup otherwise fails with `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer.
/// - `ENOTDIR`: a non-directory component was used where pathname traversal
///   required a directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
/// - `ELOOP`: current pathname resolution exceeded the kernel `MAXSYMLINKS`
///   limit while following symlinks.
/// - `EIO`: pathname resolution failed while reading a symlink target or while
///   performing filesystem lookup IO.
/// - `EOVERFLOW`: file metadata cannot be represented in the i386
///   `struct stat` layout.
///
/// Linux 1.0 also propagates filesystem-specific lookup failures from the
/// resolved filesystem. On NFS mounts this includes additional translated
/// lookup errors such as `EPERM`, `ENXIO`, `EEXIST`, `ENODEV`, `EISDIR`,
/// `EINVAL`, `EFBIG`, `ENOSPC`, `EROFS`, `ENOTEMPTY`, `EDQUOT`, and
/// `ESTALE`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v6.19#n424)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n424)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n105)
///
/// # Historical References
/// - Current i386 `struct stat`: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/include/uapi/asm/stat.h?h=v7.0#n10)
/// - Current copy-out: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0#n466)
/// - Current symlink traversal limit: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/linux/namei.h?h=v7.0#n14)
/// - Linux 1.0 `struct new_stat`, preserved as [`celer_system_linux_ctypes::linux_1_0::NewStat`]: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
pub unsafe fn stat(filename: *const Char, statbuf: *mut NewStat) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(
            Sysno::Newstat,
            filename.addr() as isize,
            statbuf.addr() as isize,
        )
    }) as Long
}

/// Get file status information through the Linux 1.0 `sys_newstat` ABI.
///
/// This wrapper uses syscall slot `106` with the Linux 1.0
/// [`Linux10NewStat`] layout from `/Volumes/External/linux/1.0/include/linux/stat.h`.
/// Current kernels use the same slot with the current i386 [`NewStat`] layout,
/// exposed by [`stat`].
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `statbuf` must point to writable memory for one [`Linux10NewStat`] value
///   for the duration of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n105)
/// - Linux 1.0 `struct new_stat`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
/// - Linux 1.0 filesystem symlink following rejected more than five nested
///   links, for example:
///   [Linux 1.0 ext2](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/ext2/symlink.c?h=1.0#n71)
pub unsafe fn stat_1_0(
    filename: *const Char,
    statbuf: *mut Linux10NewStat,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Newstat,
            filename.addr() as isize,
            statbuf.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        mem::{align_of, offset_of, size_of},
        os::unix::fs::{MetadataExt as _, symlink},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{
        Char, Long, NewStat, linux_1_0::NewStat as Linux10NewStat,
    };

    use crate::arch::{current::Sysno, linux_1_0::Sysno as Linux10Sysno};

    use super::{stat, stat_1_0};

    fn create_temp_path(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));

        path
    }

    fn zeroed_new_stat() -> NewStat {
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

    fn zeroed_linux_1_0_new_stat() -> Linux10NewStat {
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
        canary: [u8; 32],
    }

    #[test]
    fn test_stat_syscall_number() {
        assert_eq!(Sysno::Newstat as isize, 106);
    }

    #[test]
    fn test_linux_1_0_stat_syscall_number_and_layout() {
        assert_eq!(Linux10Sysno::Newstat as isize, 106);
        assert_eq!(size_of::<Linux10NewStat>(), 64);
        assert_eq!(align_of::<Linux10NewStat>(), 4);
        assert_eq!(offset_of!(Linux10NewStat, __pad1), 2);
        assert_eq!(offset_of!(Linux10NewStat, __pad2), 18);
        assert_eq!(offset_of!(Linux10NewStat, __unused1), 36);
        assert_eq!(offset_of!(Linux10NewStat, __unused2), 44);
        assert_eq!(offset_of!(Linux10NewStat, __unused3), 52);
    }

    #[test]
    fn test_new_stat_layout() {
        assert_eq!(size_of::<NewStat>(), 64);
        assert_eq!(align_of::<NewStat>(), 4);
        assert_eq!(offset_of!(NewStat, st_dev), 0);
        assert_eq!(offset_of!(NewStat, st_ino), 4);
        assert_eq!(offset_of!(NewStat, st_mode), 8);
        assert_eq!(offset_of!(NewStat, st_nlink), 10);
        assert_eq!(offset_of!(NewStat, st_uid), 12);
        assert_eq!(offset_of!(NewStat, st_gid), 14);
        assert_eq!(offset_of!(NewStat, st_rdev), 16);
        assert_eq!(offset_of!(NewStat, st_size), 20);
        assert_eq!(offset_of!(NewStat, st_blksize), 24);
        assert_eq!(offset_of!(NewStat, st_blocks), 28);
        assert_eq!(offset_of!(NewStat, st_atime), 32);
        assert_eq!(offset_of!(NewStat, st_atime_nsec), 36);
        assert_eq!(offset_of!(NewStat, st_mtime), 40);
        assert_eq!(offset_of!(NewStat, st_mtime_nsec), 44);
        assert_eq!(offset_of!(NewStat, st_ctime), 48);
        assert_eq!(offset_of!(NewStat, st_ctime_nsec), 52);
        assert_eq!(offset_of!(NewStat, __unused4), 56);
        assert_eq!(offset_of!(NewStat, __unused5), 60);
    }

    #[test]
    fn test_stat_regular_file_metadata() {
        let path = create_temp_path("celer_sys_newstat_file");
        File::create(&path).unwrap();

        let metadata = fs::metadata(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for one `NewStat` without
        // overlapping mutable aliases.
        let ret = unsafe {
            stat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "stat failed: {ret}");
        assert_eq!(statbuf.st_ino as u64, metadata.ino());
        assert_eq!(statbuf.st_mode as u32, metadata.mode());
        assert_eq!(statbuf.st_size as u64, metadata.size());
        assert_eq!(statbuf.st_blocks as u64, metadata.blocks());
        assert_eq!(statbuf.st_blksize as u64, metadata.blksize());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_stat_follows_symlink() {
        let target = create_temp_path("celer_sys_newstat_target");
        let link = create_temp_path("celer_sys_newstat_link");

        fs::write(&target, b"follow me").unwrap();
        symlink(&target, &link).unwrap();

        let target_metadata = fs::metadata(&link).unwrap();
        let link_metadata = fs::symlink_metadata(&link).unwrap();

        let mut path_bytes = link.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for one `NewStat` without
        // overlapping mutable aliases.
        let ret = unsafe {
            stat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "stat failed on symlink: {ret}");
        assert_eq!(statbuf.st_mode as u32, target_metadata.mode());
        assert_eq!(statbuf.st_size as u64, target_metadata.size());
        assert_ne!(statbuf.st_mode as u32, link_metadata.mode());

        fs::remove_file(&link).unwrap();
        fs::remove_file(&target).unwrap();
    }

    #[test]
    fn test_stat_empty_path_returns_enoent() {
        let path = b"\0";
        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path` is NUL-terminated and readable for the duration of
        // the syscall, and `statbuf` is writable for one `NewStat` without
        // overlapping mutable aliases.
        let ret =
            unsafe { stat(path.as_ptr().cast::<Char>(), &raw mut statbuf) };

        assert_eq!(ret, -(2 as Long), "expected ENOENT from stat");
    }

    #[test]
    fn test_stat_null_buffer_faults() {
        let path = b"/\0";

        // SAFETY: a null output pointer is permitted to test kernel `EFAULT`;
        // valid callers must provide writable storage for one `NewStat`.
        let ret = unsafe {
            stat(path.as_ptr().cast::<Char>(), core::ptr::null_mut())
        };

        assert_eq!(ret, -(14 as Long), "expected EFAULT from stat");
    }

    #[test]
    fn test_stat_bad_filename_pointer_faults() {
        let mut statbuf = zeroed_new_stat();

        // SAFETY: an invalid user pointer is passed deliberately to exercise
        // the kernel `EFAULT` path for `filename`.
        let ret =
            unsafe { stat(core::ptr::dangling::<Char>(), &raw mut statbuf) };

        assert_eq!(ret, -(14 as Long), "expected EFAULT from stat");
    }

    #[test]
    fn test_stat_path_too_long_returns_enametoolong() {
        let mut path = vec![b'a'; 4096];
        path.push(0);
        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path` is NUL-terminated and readable for the duration of
        // the syscall, and `statbuf` is writable for one `NewStat` without
        // overlapping mutable aliases.
        let ret =
            unsafe { stat(path.as_ptr().cast::<Char>(), &raw mut statbuf) };

        assert_eq!(ret, -(36 as Long), "expected ENAMETOOLONG from stat");
    }

    #[test]
    fn test_stat_uses_current_stat_copy_size() {
        let path = create_temp_path("celer_sys_newstat_copy_size");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = NewStatWithCanary {
            stat: zeroed_new_stat(),
            canary: [0xA5; 32],
        };

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf.stat` is writable for one `NewStat`
        // without overlapping mutable aliases.
        let ret = unsafe {
            stat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf.stat)
        };

        assert_eq!(ret, 0, "stat failed: {ret}");
        assert_eq!(statbuf.canary, [0xA5; 32], "stat overwrote canary");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_linux_1_0_stat_wrapper_success() {
        let path = create_temp_path("celer_sys_linux_1_0_stat");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_linux_1_0_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a Linux 1.0 `NewStat`.
        let ret = unsafe {
            stat_1_0(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "linux_1_0::stat failed: {ret}");

        fs::remove_file(&path).unwrap();
    }
}
