use celer_system_linux_ctypes::{Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall1};

/// Duplicate the open file descriptor `fildes`.
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
/// - On success, returns the lowest-numbered unused file descriptor in the
///   caller's file table.
/// - The new descriptor refers to the same open file description as `fildes`.
/// - The returned descriptor shares file offset and status flags with the
///   original descriptor because both refer to the same open file description.
///
/// # Errors
/// - `EBADF`: `fildes` does not refer to an open file descriptor.
/// - `EMFILE`: the caller cannot obtain another file descriptor slot.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/dup.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/fcntl.c?h=v6.19#n1477)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/fcntl.c?h=v6.18.18#n1477)
/// - x86 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n56)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/fcntl.c?h=0.10#n42)
pub fn dup(fildes: UnsignedInt) -> Int {
    // SAFETY: `dup` takes a single integer argument and has no pointer
    // validity requirements.
    unsafe { syscall1(Sysno::Dup, fildes as isize) as Int }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UnsignedInt;

    use super::dup;

    fn create_temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_dup_test_{now}"));

        path
    }

    #[test]
    fn test_dup() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        let dup_fd = dup(fd as UnsignedInt);
        assert!(dup_fd >= 0, "dup failed: {dup_fd}");
        assert_ne!(dup_fd, fd);

        crate::sys::close(dup_fd);
        crate::sys::close(fd);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_dup_invalid_fd() {
        let result = dup(!0 as UnsignedInt);

        assert_eq!(result, -9, "expected EBADF from invalid fd, got {result}");
    }
}
