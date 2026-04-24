#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::NewStat as NativeStat;
#[cfg(target_arch = "x86_64")]
use celer_system_linux_ctypes::Stat64 as NativeStat;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::NewStat as Linux10NewStat;
use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall2};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2,
};

/// Get file status information for the path named by `filename` through the
/// i386 `newlstat` ABI introduced alongside syscall slot `107`.
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
/// - Availability: always present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `statbuf` with metadata for the file resolved from
///   `filename`.
/// - The final pathname component is not followed if it is a symlink.
/// - Intermediate pathname components may still traverse symlinks during
///   resolution.
/// - This wrapper uses the current i386 `struct stat` output layout copied by
///   `cp_new_stat()`, including nanosecond timestamp fields.
///
/// # Errors
/// - `EFAULT`: `statbuf` is not writable for a full `NewStat` value, or
///   `filename` lies outside the task address space.
/// - `ENAMETOOLONG`: `filename` does not fit in the single-page kernel buffer
///   used by `getname()`.
/// - `ENOENT`: `filename` is empty, names a missing path component, or
///   pathname lookup otherwise fails with `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer
///   used by `getname()`.
/// - `ENOTDIR`: a non-directory component was used where pathname traversal
///   required a directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
/// - `EOVERFLOW`: file metadata cannot be represented in the i386
///   `struct stat` layout.
///
/// Linux 1.0 also propagates filesystem-specific errors returned by the
/// directory inode's `lookup` or `follow_link` operations.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stat.2.html)
/// - Stable: [v7.0-rc7](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0-rc7#n522)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n522)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n137)
///
/// # Historical References
/// - Current i386 `struct stat`: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/include/uapi/asm/stat.h?h=v7.0#n10)
/// - Current copy-out: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v7.0#n518)
/// - Linux 1.0 `struct new_stat`, preserved as [`celer_system_linux_ctypes::linux_1_0::NewStat`]: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
pub unsafe fn newlstat(
    filename: *const Char,
    statbuf: *mut NativeStat,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(
            Sysno::Newlstat,
            filename.addr() as isize,
            statbuf.addr() as isize,
        )
    }) as Long
}

/// Get file status information through the Linux 1.0 `sys_newlstat` ABI.
///
/// This wrapper uses syscall slot `107` with the Linux 1.0
/// [`Linux10NewStat`] layout. Current kernels use the same slot with the
/// current i386 [`NewStat`] layout, exposed by [`newlstat`].
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `statbuf` must point to writable memory for one [`Linux10NewStat`] value
///   for the duration of the syscall.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n137)
/// - Linux 1.0 `struct new_stat`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/stat.h?h=1.0#n20)
#[cfg(target_arch = "x86")]
pub unsafe fn newlstat_1_0(
    filename: *const Char,
    statbuf: *mut Linux10NewStat,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Newlstat,
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

    #[cfg(target_arch = "x86_64")]
    use celer_system_linux_ctypes::Stat64 as NativeStat;
    use celer_system_linux_ctypes::{Char, Stat};
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::{
        NewStat as NativeStat, linux_1_0::NewStat as Linux10NewStat,
    };

    use crate::arch::current::Sysno;
    #[cfg(target_arch = "x86")]
    use crate::arch::linux_1_0::Sysno as Linux10Sysno;

    use super::newlstat;
    #[cfg(target_arch = "x86")]
    use super::newlstat_1_0;

    fn create_temp_path(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));

        path
    }

    fn zeroed_new_stat() -> NativeStat {
        NativeStat {
            st_dev: 0,
            st_ino: 0,
            #[cfg(target_arch = "x86_64")]
            st_nlink: 0,
            st_mode: 0,
            #[cfg(target_arch = "x86")]
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            #[cfg(target_arch = "x86_64")]
            __pad0: 0,
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
            #[cfg(target_arch = "x86")]
            __unused4: 0,
            #[cfg(target_arch = "x86")]
            __unused5: 0,
            #[cfg(target_arch = "x86_64")]
            __unused: [0; 3],
        }
    }

    #[cfg(target_arch = "x86")]
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
        stat: NativeStat,
        canary: [u8; 32],
    }

    #[test]
    fn test_newlstat_symlink_metadata() {
        let target = create_temp_path("celer_sys_newlstat_target");
        let link = create_temp_path("celer_sys_newlstat_link");

        File::create(&target).unwrap();
        symlink(&target, &link).unwrap();

        let link_metadata = fs::symlink_metadata(&link).unwrap();
        let target_metadata = fs::metadata(&link).unwrap();

        let mut path_bytes = link.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a full `NewStat`
        // without overlapping mutable aliases.
        let ret = unsafe {
            newlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "newlstat failed: {ret}");
        #[cfg(target_arch = "x86")]
        assert_eq!(statbuf.st_ino as u64, link_metadata.ino());
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        assert_eq!(statbuf.st_ino, link_metadata.ino());
        #[cfg(target_arch = "x86")]
        assert_eq!(statbuf.st_mode as u32, link_metadata.mode());
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        assert_eq!(statbuf.st_mode, link_metadata.mode());
        assert_eq!(statbuf.st_size as u64, link_metadata.size());
        #[cfg(target_arch = "x86")]
        assert_ne!(statbuf.st_mode as u32, target_metadata.mode());
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        assert_ne!(statbuf.st_mode, target_metadata.mode());

        fs::remove_file(&link).unwrap();
        fs::remove_file(&target).unwrap();
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_newlstat_wrapper_success() {
        let path = create_temp_path("celer_sys_linux_1_0_newlstat");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_linux_1_0_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a Linux 1.0 `NewStat`.
        let ret = unsafe {
            newlstat_1_0(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "linux_1_0::newlstat failed: {ret}");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_newlstat_empty_path() {
        let path_bytes = [0_u8];
        let mut statbuf = zeroed_new_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a full `NewStat`
        // without overlapping mutable aliases.
        let ret = unsafe {
            newlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, -2, "expected ENOENT from newlstat, got {ret}");
    }

    #[test]
    fn test_newlstat_null_statbuf_faults() {
        let path = create_temp_path("celer_sys_newlstat_null_statbuf");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and passing a null output pointer is permitted by
        // the kernel ABI for this negative test.
        let ret = unsafe {
            newlstat(path_bytes.as_ptr().cast::<Char>(), core::ptr::null_mut())
        };

        assert_eq!(ret, -14, "expected EFAULT from newlstat, got {ret}");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_newlstat_uses_current_stat_copy_size() {
        let path = create_temp_path("celer_sys_newlstat_copy_size");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = NewStatWithCanary {
            stat: zeroed_new_stat(),
            canary: [0xA5; 32],
        };

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf.stat` is writable for a full
        // `NewStat` without overlapping mutable aliases.
        let ret = unsafe {
            newlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf.stat)
        };

        assert_eq!(ret, 0, "newlstat failed: {ret}");
        assert!(size_of::<NativeStat>() > size_of::<Stat>());
        assert_eq!(statbuf.canary, [0xA5; 32], "newlstat overwrote canary");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_newlstat_abi_layout() {
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(Sysno::Newlstat as isize, 107);
            assert_eq!(Linux10Sysno::Newlstat as isize, 107);
            assert_eq!(size_of::<NativeStat>(), 64);
            assert_eq!(align_of::<NativeStat>(), 4);
            assert_eq!(offset_of!(NativeStat, st_ino), 4);
            assert_eq!(offset_of!(NativeStat, st_size), 20);
            assert_eq!(offset_of!(NativeStat, st_blksize), 24);
            assert_eq!(offset_of!(NativeStat, st_blocks), 28);
            assert_eq!(offset_of!(NativeStat, st_ctime), 48);
            assert_eq!(offset_of!(NativeStat, st_ctime_nsec), 52);
        }
        #[cfg(target_arch = "x86_64")]
        {
            assert_eq!(Sysno::Newlstat as isize, 6);
            assert_eq!(size_of::<NativeStat>(), 144);
            assert_eq!(align_of::<NativeStat>(), 8);
            assert_eq!(offset_of!(NativeStat, st_ino), 8);
            assert_eq!(offset_of!(NativeStat, st_size), 48);
            assert_eq!(offset_of!(NativeStat, st_blksize), 56);
            assert_eq!(offset_of!(NativeStat, st_blocks), 64);
            assert_eq!(offset_of!(NativeStat, st_ctime), 104);
            assert_eq!(offset_of!(NativeStat, st_ctime_nsec), 112);
        }
    }
}
