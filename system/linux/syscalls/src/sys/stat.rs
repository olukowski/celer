use celer_system_linux_ctypes::{Char, Long, Stat};

use crate::arch::current::{Sysno, syscall2};

/// Get file status information for the path named by `filename`.
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - `statbuf` must point to writable memory large enough for a `Stat` value.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `statbuf` with metadata for the file resolved from
///   `filename`.
/// - This wrapper follows the kernel's `stat` ABI.
///
/// # Errors
/// - The syscall returns the same errors as the underlying filesystem and
///   path-resolution code.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/stat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v6.19#n424)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n424)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=1.0#n85)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=0.10#n14)
pub unsafe fn stat(filename: *const Char, statbuf: *mut Stat) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(
            Sysno::Stat,
            filename.addr() as isize,
            statbuf.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::unix::fs::MetadataExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, Stat};

    use super::stat;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_stat_{now}"));

        path
    }

    #[test]
    fn test_stat() {
        let path = create_temp_path();
        File::create(&path).unwrap();

        let metadata = fs::metadata(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let mut statbuf = Stat {
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
        };

        // SAFETY: `path_bytes` is NUL-terminated and readable for the duration
        // of the syscall, and `statbuf` is writable for a full `Stat`.
        let ret = unsafe {
            stat(
                path_bytes.as_ptr().cast::<Char>(),
                &raw mut statbuf,
            )
        };

        assert_eq!(ret, 0, "stat failed: {ret}");
        assert_eq!(statbuf.st_ino as u64, metadata.ino());
        assert_eq!(statbuf.st_size as u64, metadata.size());

        fs::remove_file(&path).unwrap();
    }
}
