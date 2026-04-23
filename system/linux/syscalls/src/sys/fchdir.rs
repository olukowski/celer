use celer_system_linux_ctypes::{Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall1};

/// Change the calling process's current working directory to the directory
/// referenced by `fd`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 checked the descriptor table, included a
///   null-inode `ENOENT` branch whose syscall reachability is unverified, then
///   validated directory type and execute permission before swapping
///   `current->pwd`; current kernels additionally require a directory dentry
///   and route the permission check through
///   `file_permission(..., MAY_EXEC | MAY_CHDIR)`.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `fd` must name an already open file descriptor in the caller's file
///   table.
/// - Linux 1.0 uses the descriptor's inode directly; no pathname lookup or
///   symlink traversal occurs in the syscall body.
/// - Linux 1.0 drops the previous working-directory inode reference, stores
///   the new directory inode as `current->pwd`, and increments that inode's
///   reference count.
/// - On success, returns `0`.
///
/// # Errors
/// - Linux 1.0 direct entry-path errors:
///   - `EBADF`: `fd` is outside the open-file table or does not name an open
///     file descriptor.
///   - `ENOTDIR`: `fd` refers to an inode that is not a directory.
///   - `EACCES`: the referenced directory fails the execute-permission check.
///
/// Linux 1.0 contains an additional `ENOENT` branch if the file table entry's
/// inode pointer is null, but that report has not been verified as reachable
/// from a real syscall entry path and is therefore not documented as a syscall
/// error here.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fchdir.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v7.0#n573)
/// - Stable table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n148)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n581)
/// - LTS table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n148)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n213)
///
/// # Historical References
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n142)
pub fn fchdir(fd: UnsignedInt) -> Int {
    // SAFETY: `fchdir` takes only an integer file descriptor and has no
    // caller-visible memory-safety preconditions.
    unsafe { syscall1(Sysno::Fchdir, fd as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        fs::OpenOptions,
        os::fd::AsRawFd as _,
        os::unix::fs::PermissionsExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UnsignedInt;

    use crate::sys::test_support::process_global_state_guard;
    use crate::{arch::current::Sysno, sys::geteuid16};

    use super::fchdir;

    fn create_temp_dir(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_fchdir_{label}_{now}"));
        fs::create_dir(&path).unwrap();

        path
    }

    #[test]
    fn test_fchdir_sysno() {
        assert_eq!(Sysno::Fchdir as isize, 133);
    }

    #[test]
    fn test_fchdir_changes_cwd() {
        let _guard = process_global_state_guard();
        let original_dir = env::current_dir().unwrap();
        let path = create_temp_dir("success");
        let dir = OpenOptions::new().read(true).open(&path).unwrap();

        let result = fchdir(dir.as_raw_fd() as UnsignedInt);
        assert_eq!(result, 0, "fchdir failed: {result}");
        assert_eq!(env::current_dir().unwrap(), path);

        let mut original_dir_bytes =
            original_dir.as_os_str().as_encoded_bytes().to_vec();
        original_dir_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let restore =
            unsafe { crate::sys::chdir(original_dir_bytes.as_ptr().cast()) };
        assert_eq!(restore, 0, "restoring cwd failed: {restore}");

        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn test_fchdir_invalid_fd() {
        let result = fchdir(UnsignedInt::MAX);

        assert_eq!(result, -9, "expected EBADF from invalid fd, got {result}");
    }

    #[test]
    fn test_fchdir_rejects_regular_file() {
        let path = env::temp_dir().join(format!(
            "celer_sys_fchdir_file_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let result = fchdir(file.as_raw_fd() as UnsignedInt);
        assert_eq!(
            result, -20,
            "expected ENOTDIR from regular file fd, got {result}"
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchdir_requires_execute_permission() {
        if geteuid16() == 0 {
            return;
        }

        let _guard = process_global_state_guard();
        let path = create_temp_dir("eacces");
        let dir = OpenOptions::new().read(true).open(&path).unwrap();
        let original_mode = fs::metadata(&path).unwrap().permissions().mode();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&path, permissions).unwrap();

        let result = fchdir(dir.as_raw_fd() as UnsignedInt);
        assert_eq!(
            result, -13,
            "expected EACCES from non-executable directory, got {result}"
        );

        fs::set_permissions(&path, fs::Permissions::from_mode(original_mode))
            .unwrap();
        fs::remove_dir(&path).unwrap();
    }
}
