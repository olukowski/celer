use celer_system_linux_ctypes::{Long, Utimbuf};

use crate::arch::current::{Sysno, syscall2};

/// Update the access and modification timestamps of a file by pathname.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None for the syscall itself.
/// - Explicit timestamp updates may still fail if the caller lacks ownership or
///   equivalent capability for the target inode.
///
/// # Behavior
/// - If `times` is null, the kernel sets both timestamps to the current time.
/// - If `times` is non-null, the kernel copies `actime` and `modtime` seconds
///   from user space and sets both nanosecond fields to zero internally.
/// - This wrapper uses the x86 legacy `utime` pathname ABI, which maps to the
///   kernel's old 32-bit-time entry.
///
/// # Errors
/// - `EFAULT`: `pathname` or `times` is not readable for the required data.
/// - `EPERM`: the caller is not permitted to set explicit timestamps.
/// - `EROFS`: the target mount is read-only.
/// - `ENOENT`, `ENAMETOOLONG`, `EACCES`, `ENOTDIR`, `ESTALE`: pathname lookup
///   failures from the VFS path walk.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/utime.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/utimes.c?h=v6.19#n211)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/utimes.c?h=v6.18.18#n211)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n24)
pub fn utime(
    pathname: *const celer_system_linux_ctypes::Char,
    times: *const Utimbuf,
) -> Long {
    // SAFETY: this wrapper forwards the raw user pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall2(
            Sysno::Utime,
            pathname.addr() as isize,
            times.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::unix::fs::MetadataExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, TimeT, Utimbuf};

    use super::utime;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_sys_utime_{now}"));
        path
    }

    #[test]
    fn test_utime() {
        let path = temp_path();
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let times = Utimbuf {
            actime: 1 as TimeT,
            modtime: 2 as TimeT,
        };

        let ret = utime(path_bytes.as_ptr().cast::<Char>(), &times);
        assert_eq!(ret, 0, "utime failed: {ret}");

        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.atime(), 1);
        assert_eq!(metadata.mtime(), 2);

        fs::remove_file(&path).unwrap();
    }
}
