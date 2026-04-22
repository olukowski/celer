use celer_system_linux_ctypes::{Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Duplicate `oldfd` onto the exact descriptor number `newfd`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: current kernels can additionally return `EBUSY` from a
///   descriptor-table race; Linux 1.0 did not expose that path.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, returns `newfd`.
/// - If `oldfd == newfd`, validates `oldfd` and returns it unchanged.
/// - For distinct descriptors, Linux 1.0 closes `newfd` first and then
///   duplicates `oldfd` into that slot.
/// - The returned descriptor refers to the same open file description as
///   `oldfd`, so file offset and status flags are shared.
///
/// # Errors
/// - `EBADF`: `oldfd` is not open, or `newfd` is at or above Linux 1.0's
///   `NR_OPEN` limit.
///
/// Current kernels may also return additional errors such as `EBUSY`; those
/// later-kernel outcomes are not exhaustively documented here.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/dup.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/file.c?h=v6.19#n1464)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/file.c?h=v6.18.18#n1464)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/fcntl.c?h=1.0#n38)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/fcntl.c?h=0.10#n36)
pub fn dup2(oldfd: UnsignedInt, newfd: UnsignedInt) -> Int {
    // SAFETY: `dup2` takes only integer arguments and has no caller-visible
    // memory-safety precondition.
    unsafe { syscall2(Sysno::Dup2, oldfd as isize, newfd as isize) as Int }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        os::unix::fs::MetadataExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Stat, UnsignedInt};

    use super::dup2;

    fn create_temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_dup2_test_{now}"));

        path
    }

    #[test]
    fn test_dup2() {
        let old_path = create_temp_path();
        let new_path = create_temp_path();
        let old_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&old_path)
            .unwrap();
        let new_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&new_path)
            .unwrap();

        let fd = old_file.into_raw_fd();
        let newfd = new_file.into_raw_fd();

        let mut old_stat = Stat {
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
        let mut new_stat = old_stat;

        let old_metadata = fs::metadata(&old_path).unwrap();
        let new_metadata = fs::metadata(&new_path).unwrap();
        assert_ne!(old_metadata.ino(), new_metadata.ino());

        let old_stat_ret = unsafe {
            crate::sys::oldfstat(fd as UnsignedInt, &raw mut old_stat)
        };
        let new_stat_ret = unsafe {
            crate::sys::oldfstat(newfd as UnsignedInt, &raw mut new_stat)
        };
        assert_eq!(
            old_stat_ret, 0,
            "oldfstat failed for old fd: {old_stat_ret}"
        );
        assert_eq!(
            new_stat_ret, 0,
            "oldfstat failed for new fd: {new_stat_ret}"
        );
        assert_eq!(old_stat.st_ino as u64, old_metadata.ino());
        assert_eq!(new_stat.st_ino as u64, new_metadata.ino());

        let ret = dup2(fd as UnsignedInt, newfd as UnsignedInt);
        assert_eq!(ret, newfd, "dup2 failed: {ret}");

        let replaced_stat = unsafe {
            crate::sys::oldfstat(newfd as UnsignedInt, &raw mut new_stat)
        };
        assert_eq!(
            replaced_stat, 0,
            "oldfstat failed after dup2: {replaced_stat}"
        );
        assert_eq!(new_stat.st_ino as u64, old_metadata.ino());

        assert_eq!(crate::sys::close(newfd), 0);
        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&old_path).unwrap();
        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_dup2_same_fd() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        let ret = dup2(fd as UnsignedInt, fd as UnsignedInt);

        assert_eq!(ret, fd);
        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_dup2_invalid_oldfd() {
        let ret = dup2(!0 as UnsignedInt, 0);

        assert_eq!(ret, -9, "expected EBADF from invalid fd, got {ret}");
    }

    #[test]
    fn test_dup2_invalid_same_fd() {
        let ret = dup2(!0 as UnsignedInt, !0 as UnsignedInt);

        assert_eq!(ret, -9, "expected EBADF from invalid fd, got {ret}");
    }

    #[test]
    fn test_dup2_invalid_newfd_boundary() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        let ret = dup2(fd as UnsignedInt, 256 as UnsignedInt);

        assert_eq!(
            ret, -9,
            "expected EBADF from out-of-range newfd, got {ret}"
        );
        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }
}
