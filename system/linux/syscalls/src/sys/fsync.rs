use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall1};

/// Flush pending file data and metadata for the open file referenced by `fd`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 collapsed any nonzero filesystem `fsync`
///   callback result to `EIO`; current kernels return the filesystem
///   callback's errno unchanged.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `fd` names an already open file descriptor; this syscall does not perform
///   path lookup.
/// - Linux 1.0 rejects descriptors whose file operations table does not
///   provide an `fsync` callback.
/// - On success, returns `0`.
///
/// # Errors
/// - `EBADF`: `fd` is outside the open-file table or does not name an open
///   file descriptor.
/// - `EINVAL`: the open file has no file-operations table, or that table has
///   no `fsync` callback.
/// - `EIO`: Linux 1.0 mapped any nonzero filesystem `fsync` callback failure
///   to `EIO`.
///
/// Current kernels may additionally return filesystem-specific errno values
/// directly from the active `fsync` callback.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fsync.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/sync.c?h=v7.0#n214)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n133)
/// - Stable x86_64 table: [v7.0 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n86)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n218)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/sync.c?h=v6.18.18#n215)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n133)
/// - LTS x86_64 table: [v6.18.18 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v6.18.18#n86)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n218)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/buffer.c?h=1.0#n177)
///
/// # Historical References
/// - Linux 1.0 syscall number: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n127)
pub fn fsync(fd: Int) -> Int {
    // SAFETY: `fsync` takes a single scalar argument and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall1(Sysno::Fsync, fd as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::fd::{AsRawFd as _, IntoRawFd as _},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Int;

    use crate::arch::current::Sysno;

    use super::fsync;

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_fsync_test_{now}"));

        path
    }

    #[test]
    fn test_fsync_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Fsync as isize, 118);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Fsync as isize, 82);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(Sysno::Fsync as isize, 74);
    }

    #[test]
    fn test_fsync() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        file.write_all(b"hello fsync").unwrap();

        let result = fsync(file.as_raw_fd() as Int);
        assert_eq!(result, 0, "fsync failed: {result}");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fsync_invalid_fd() {
        let result = fsync(-1);

        assert_eq!(result, -9, "expected EBADF from invalid fd, got {result}");
    }

    #[test]
    fn test_fsync_pipe_returns_einval() {
        let mut fds = [0 as Int; 2];

        // SAFETY: `fds` is writable for two `Int` values.
        let rc = unsafe { crate::sys::test_support::pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");

        let result = fsync(fds[0]);
        assert_eq!(
            result, -22,
            "expected EINVAL from pipe fsync, got {result}"
        );

        assert_eq!(crate::sys::close(fds[0]), 0);
        assert_eq!(crate::sys::close(fds[1]), 0);
    }

    #[test]
    fn test_fsync_closed_fd_returns_ebadf() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let fd = file.into_raw_fd();

        assert_eq!(crate::sys::close(fd), 0);

        let result = fsync(fd);
        assert_eq!(result, -9, "expected EBADF from closed fd, got {result}");

        fs::remove_file(&path).unwrap();
    }
}
