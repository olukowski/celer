use celer_system_linux_ctypes::{Char, Long, Stat};

use crate::arch::current::{Sysno, syscall2};

/// Get file status information for the path named by `filename` through the
/// legacy i386 `oldlstat` ABI.
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `statbuf` must point to writable memory large enough for a `Stat` value,
///   and no other pointer or reference may alias that output for mutable
///   access for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 1.0 kept this legacy ABI at syscall slot `84`
///   after adding the newer `lstat` entry at syscall slot `107`.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `statbuf` with metadata for the file resolved from
///   `filename`.
/// - The final pathname component is not followed if it is a symlink.
/// - This wrapper follows the kernel's legacy i386 `oldlstat` ABI at syscall
///   slot `84`.
///
/// # Errors
/// - `EFAULT`: `statbuf` lies outside the task address space, or `filename`
///   points outside the task address space.
/// - `ENAMETOOLONG`: `filename` does not fit in the single-page kernel buffer
///   used by `getname()`.
/// - `ENOENT`: `filename` is empty, names a missing path component, or
///   pathname lookup otherwise fails with `ENOENT`.
/// - `ENOMEM`: the kernel could not allocate the temporary pathname buffer
///   used by `getname()`.
/// - `ENOTDIR`: a non-directory component was used where pathname traversal
///   required a directory.
/// - `EACCES`: pathname traversal lacked search permission on a directory.
///
/// The historical Linux 1.0 `sys_lstat` entry also propagates filesystem
/// lookup errors returned by the directory inode's `lookup` operation.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v6.19#n433)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n433)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n121)
///
/// # Historical References
/// - First appearance: [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=0.12#n47)
pub unsafe fn oldlstat(filename: *const Char, statbuf: *mut Stat) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(
            Sysno::Oldlstat,
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
        mem::size_of,
        os::unix::fs::{MetadataExt as _, symlink},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, Stat};

    use super::oldlstat;

    fn create_temp_path(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));

        path
    }

    fn zeroed_stat() -> Stat {
        Stat {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_size: 0,
            st_atime: 0,
            st_mtime: 0,
            st_ctime: 0,
        }
    }

    #[repr(C)]
    struct StatWithCanary {
        stat: Stat,
        canary: [u8; 64],
    }

    #[repr(C)]
    struct Linux1NewStat {
        st_dev: u16,
        __pad1: u16,
        st_ino: u32,
        st_mode: u16,
        st_nlink: u16,
        st_uid: u16,
        st_gid: u16,
        st_rdev: u16,
        __pad2: u16,
        st_size: u32,
        st_blksize: u32,
        st_blocks: u32,
        st_atime: u32,
        __unused1: u32,
        st_mtime: u32,
        __unused2: u32,
        st_ctime: u32,
        __unused3: u32,
        __unused4: u32,
        __unused5: u32,
    }

    #[test]
    fn test_oldlstat_symlink_metadata() {
        let target = create_temp_path("celer_sys_oldlstat_target");
        let link = create_temp_path("celer_sys_oldlstat_link");

        File::create(&target).unwrap();
        symlink(&target, &link).unwrap();

        let link_metadata = fs::symlink_metadata(&link).unwrap();
        let target_metadata = fs::metadata(&link).unwrap();

        let mut path_bytes = link.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a full `Stat` without
        // overlapping mutable aliases.
        let ret = unsafe {
            oldlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, 0, "oldlstat failed: {ret}");
        assert_eq!(statbuf.st_mode as u32, link_metadata.mode());
        assert_eq!(statbuf.st_size as u64, link_metadata.size());
        assert_ne!(statbuf.st_mode as u32, target_metadata.mode());

        fs::remove_file(&link).unwrap();
        fs::remove_file(&target).unwrap();
    }

    #[test]
    fn test_oldlstat_missing_path() {
        let path = create_temp_path("celer_sys_oldlstat_missing");

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = zeroed_stat();

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a full `Stat` without
        // overlapping mutable aliases.
        let ret = unsafe {
            oldlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf)
        };

        assert_eq!(ret, -2, "expected ENOENT from oldlstat, got {ret}");
    }

    #[test]
    fn test_oldlstat_uses_legacy_copy_size() {
        let path = create_temp_path("celer_sys_oldlstat_copy_size");
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = StatWithCanary {
            stat: zeroed_stat(),
            canary: [0xA5; 64],
        };

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf.stat` is writable for a full `Stat`
        // without overlapping mutable aliases.
        let ret = unsafe {
            oldlstat(path_bytes.as_ptr().cast::<Char>(), &raw mut statbuf.stat)
        };

        assert_eq!(ret, 0, "oldlstat failed: {ret}");
        assert!(size_of::<Stat>() < size_of::<Linux1NewStat>());
        assert!(size_of::<StatWithCanary>() >= size_of::<Linux1NewStat>());
        assert_eq!(statbuf.canary, [0xA5; 64], "oldlstat overwrote canary");

        fs::remove_file(&path).unwrap();
    }
}
