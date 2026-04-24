use celer_system_linux_ctypes::{Int, OffT, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall2 as linux_1_0_syscall2,
};

/// Truncate the open file referred to by `fd` to `length` bytes.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepted an unsigned length and only checked
///   the descriptor, directory status, and write mode in the syscall body;
///   current kernels add earlier validation for file type, append-only state,
///   signed offsets, and security hooks before the truncate path runs.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `fd` names an already open file descriptor; this syscall does not perform
///   path lookup.
/// - Linux 1.0 updates the inode size immediately, runs the filesystem
///   truncate hook if present, refreshes `ctime` and `mtime`, and then forwards
///   to `notify_change(NOTIFY_SIZE, inode)`.
/// - This wrapper uses the current x86 `off_t`-shaped ABI for `length`, even
///   though Linux 1.0 implemented `sys_ftruncate` with an `unsigned int`
///   length parameter.
///
/// # Errors
/// - Linux 1.0 direct entry-path errors:
///   - `EBADF`: `fd` is outside the open-file table or does not name an open
///     file descriptor.
///   - `ENOENT`: the file table entry has no inode attached.
///   - `EACCES`: the descriptor is not open for writing, or the target inode
///     is a directory.
/// - Current kernels additionally return:
///   - `EINVAL`: `length` is negative, the file is not a regular file, the
///     descriptor is not open for writing, or the request exceeds the legacy
///     non-large-file limit for this entrypoint.
///   - `EPERM`: the inode is append-only.
///
/// Filesystem-specific truncate and attribute-change hooks may return
/// additional errors after the direct checks above. In Linux 1.0, the
/// verified nontrivial forwarded set comes from NFS attribute updates.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/truncate.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n202)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n202)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n94)
///
/// # Historical References
/// - First appearance: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n102)
pub fn ftruncate(fd: UnsignedInt, length: OffT) -> Int {
    // SAFETY: the syscall only takes integer arguments and has no caller-side
    // memory-safety precondition.
    unsafe { syscall2(Sysno::Ftruncate, fd as isize, length as isize) as Int }
}

/// Truncate the open file referred to by `fd` through the Linux 1.0 unsigned
/// `ftruncate` ABI.
///
/// This is the historical Linux 1.0 ABI at syscall slot `93`, which took
/// `length` as an `unsigned int`. Current x86 kernels use the same syscall
/// number for the signed current ABI exposed by [`ftruncate`].
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n94)
/// - Current implementation:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v7.0#n211)
#[cfg(target_arch = "x86")]
pub fn ftruncate_1_0(fd: UnsignedInt, length: UnsignedInt) -> Int {
    // SAFETY: the syscall only takes integer arguments and has no caller-side
    // memory-safety precondition.
    unsafe {
        linux_1_0_syscall2(
            Linux10Sysno::Ftruncate,
            fd as isize,
            length as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OffT, UnsignedInt};

    use crate::arch::{current::Sysno, linux_1_0::Sysno as Linux10Sysno};

    use super::{ftruncate, ftruncate_1_0};

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_ftruncate_{now}"));

        path
    }

    #[test]
    fn test_ftruncate_sysno() {
        assert_eq!(Sysno::Ftruncate as isize, 93);
    }

    #[test]
    fn test_ftruncate_1_0_sysno() {
        assert_eq!(Linux10Sysno::Ftruncate as isize, 93);
    }

    #[test]
    fn test_ftruncate_changes_file_length() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        file.write_all(b"Hello, World!").unwrap();

        let ret = ftruncate(file.as_raw_fd() as UnsignedInt, 5 as OffT);
        assert_eq!(ret, 0, "ftruncate failed: {ret}");
        assert_eq!(fs::metadata(&path).unwrap().len(), 5);

        let ret = ftruncate(file.as_raw_fd() as UnsignedInt, 16 as OffT);
        assert_eq!(ret, 0, "ftruncate failed: {ret}");
        assert_eq!(fs::metadata(&path).unwrap().len(), 16);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_ftruncate_invalid_fd() {
        let ret = ftruncate(9_999 as UnsignedInt, 0 as OffT);
        assert_eq!(ret, -9);
    }

    #[test]
    fn test_ftruncate_1_0_invalid_fd() {
        let ret = ftruncate_1_0(9_999 as UnsignedInt, 0 as UnsignedInt);
        assert_eq!(ret, -9);
    }

    #[test]
    fn test_ftruncate_negative_length() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let ret = ftruncate(file.as_raw_fd() as UnsignedInt, -1 as OffT);
        assert_eq!(ret, -22);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_ftruncate_read_only_fd() {
        let path = create_temp_path();
        fs::write(&path, b"Hello, World!").unwrap();

        let file = OpenOptions::new().read(true).open(&path).unwrap();

        let ret = ftruncate(file.as_raw_fd() as UnsignedInt, 5 as OffT);
        assert_eq!(ret, -22);

        fs::remove_file(&path).unwrap();
    }
}
