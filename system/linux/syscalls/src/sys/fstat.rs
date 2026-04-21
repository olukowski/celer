use celer_system_linux_ctypes::{Long, Stat, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Get file status information for an open file descriptor.
///
/// # Safety
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
/// - On success, fills `statbuf` with metadata for the open file referenced by
///   `fd`.
/// - This wrapper follows the kernel's legacy 32-bit `fstat` ABI.
/// - That legacy ABI can return `EOVERFLOW` when the file metadata cannot be
///   represented in `Stat`.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `EOVERFLOW`: the file metadata cannot be represented in `Stat`.
/// - `EFAULT`: `statbuf` is not writable for a full `Stat` value.
/// - Other filesystem and VFS errors may be returned by the underlying file
///   lookup and copy-out paths.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fstat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/stat.c?h=v6.19#n450)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/stat.c?h=v6.18.18#n450)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n43)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/stat.c?h=0.10#n47)
pub unsafe fn fstat(fd: UnsignedInt, statbuf: *mut Stat) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall2(Sysno::Fstat, fd as isize, statbuf.addr() as isize) })
        as Long
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        os::unix::fs::MetadataExt as _,
    };

    use celer_system_linux_ctypes::{Stat, UnsignedInt};

    use crate::sys::close;

    use super::fstat;

    #[test]
    fn test_fstat() {
        let path = std::env::temp_dir().join("celer_sys_fstat_test");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let metadata = file.metadata().unwrap();
        let fd = file.into_raw_fd();

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

        // SAFETY: `statbuf` is writable for a full `Stat`.
        let ret = unsafe { fstat(fd as UnsignedInt, &raw mut statbuf) };
        assert_eq!(ret, 0, "fstat failed: {ret}");
        assert_eq!(statbuf.st_ino as u64, metadata.ino());
        assert_eq!(statbuf.st_size as u64, metadata.size());

        assert_eq!(close(fd), 0);

        let bad = unsafe { fstat(fd as UnsignedInt, &raw mut statbuf) };
        assert_eq!(bad, -9, "expected EBADF from closed fd, got {bad}");

        fs::remove_file(&path).unwrap();
    }
}
