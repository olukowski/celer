use celer_system_linux_ctypes::{Long, OffT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Reposition the file offset for the open file descriptor `fd`.
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
/// - On success, returns the resulting file offset.
/// - `whence` selects how `offset` is interpreted.
/// - For invalid file descriptors or unsupported seek modes, the kernel
///   returns a negative errno value.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `EINVAL`: `whence` is not a supported seek mode.
/// - `EOVERFLOW`: the resulting offset cannot be represented in the syscall
///   return type on 32-bit platforms.
///
/// Other errors may occur depending on the underlying file object and kernel
/// configuration.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/lseek.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v6.19#n393)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n393)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n37)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.10#n25)
pub fn lseek(fd: UnsignedInt, offset: OffT, whence: UnsignedInt) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(Sysno::Lseek, fd as isize, offset as isize, whence as isize)
    }) as Long
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::{Seek as _, SeekFrom, Write as _},
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OffT, UnsignedInt};

    use super::lseek;

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_lseek_{now}"));

        path
    }

    #[test]
    fn test_lseek() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();

        file.write_all(b"Hello, World!").unwrap();
        file.rewind().unwrap();

        let expected = file.seek(SeekFrom::Current(0)).unwrap() as OffT;

        // SAFETY: `file.as_raw_fd()` is a valid open descriptor and `SEEK_CUR`
        // is a supported seek mode.
        let actual = lseek(file.as_raw_fd() as UnsignedInt, 0 as OffT, 1);

        assert_eq!(actual, expected as _);

        fs::remove_file(&path).unwrap();
    }
}
