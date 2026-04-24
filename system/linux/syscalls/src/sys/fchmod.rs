use celer_system_linux_ctypes::{Int, UModeT};

use crate::arch::current::{Sysno, syscall2};

/// Change the mode bits of the file referenced by `fd`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 treated `mode == (mode_t)-1` as "preserve
///   the current mode bits"; current kernels do not keep that special
///   case.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Linux 1.0 looks up `fd` in the calling task's file table and applies the
///   mode change to the referenced inode.
/// - Linux 1.0 preserves the inode's file type bits and replaces the mode bits
///   covered by `S_IALLUGO` (`setuid`, `setgid`, `sticky`, and `0o777`).
/// - Linux 1.0 clears `S_ISGID` after the update when the caller is neither
///   superuser nor a member of the file's group.
/// - On success, returns `0`.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `ENOENT`: `fd` refers to a file table entry whose inode pointer is null.
/// - `EPERM`: the caller is neither the inode owner nor superuser.
/// - `EROFS`: the inode is on a read-only filesystem.
///
/// Filesystem-specific `notify_change` hooks may return additional errno
/// values. In Linux 1.0 this is especially relevant for NFS-backed inodes.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fchmod.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n657)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n657)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n253)
///
/// # Historical References
/// - First appearance: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n253)
pub fn fchmod(fd: Int, mode: UModeT) -> Int {
    // SAFETY: `fchmod` takes only scalar arguments and has no caller-visible
    // memory-safety preconditions.
    unsafe { syscall2(Sysno::Fchmod, fd as isize, mode as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        os::unix::fs::PermissionsExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Int, UModeT};

    use crate::arch::current::Sysno;

    use super::fchmod;

    fn create_temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_fchmod_test_{now}"));

        path
    }

    #[test]
    fn test_fchmod_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 94;
        #[cfg(target_arch = "aarch64")]
        let expected = 52;

        assert_eq!(Sysno::Fchmod as isize, expected);
    }

    #[test]
    fn test_fchmod() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let fd = file.into_raw_fd();

        let result = fchmod(fd, 0o600 as UModeT);
        assert_eq!(result, 0, "fchmod failed: {result}");

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchmod_preserves_special_mode_bits() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let fd = file.into_raw_fd();

        let requested_mode = 0o1600 as UModeT;
        let result = fchmod(fd, requested_mode);
        assert_eq!(result, 0, "fchmod failed: {result}");

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, requested_mode.into());

        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchmod_invalid_fd() {
        let result = fchmod(-1 as Int, 0o600 as UModeT);

        assert_eq!(result, -9, "expected EBADF from invalid fd, got {result}");
    }
}
