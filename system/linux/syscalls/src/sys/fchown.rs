use celer_system_linux_ctypes::{Int, OldGidT, OldUidT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Change the owner and/or group of an open file descriptor through the legacy
/// i386 `fchown16` ABI.
///
/// Pass `OldUidT::MAX` and/or `OldGidT::MAX` to preserve the existing owner
/// or group respectively.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 exposed this entry as `sys_fchown`; current
///   i386 kernels keep syscall `95` as the legacy `fchown16` wrapper, which
///   widens the 16-bit IDs before delegating to the modern ownership-change
///   path
/// - Availability: current x86 32-bit kernels expose the legacy
///   `sys_fchown16` entry point when `CONFIG_UID16` is enabled
///
/// # Required Privileges
/// - None on Linux 1.0 when the caller already owns the inode, requests that
///   same owner, and requests either the inode's current group or a group in
///   the caller's effective or supplementary group set.
/// - Current kernels keep the same basic unprivileged shape: without
///   `CAP_CHOWN`, the caller must own the inode; from there, owner changes are
///   limited to preserving the current owner, while group changes may target
///   either the current group or one of the caller's supplementary groups.
/// - `CAP_CHOWN` is required on current kernels outside those owner-owned
///   paths.
///
/// # Behavior
/// - `fd` names an already open file descriptor; this syscall does not perform
///   path lookup or symlink traversal.
/// - Linux 1.0 treats `OldUidT::MAX` and `OldGidT::MAX` as "preserve the
///   current inode owner/group" sentinels before permission checks run.
/// - Linux 1.0 rejects the call with `EROFS` before the ownership checks if
///   the target inode lives on a read-only filesystem.
/// - On the permitted path, Linux 1.0 updates the inode owner, group, and
///   `ctime`, marks the inode dirty, and then forwards to
///   `notify_change(NOTIFY_UIDGID, inode)`.
/// - Successful calls return `0`.
///
/// # Errors
/// - Linux 1.0 direct entry-path errors:
///   - `EBADF`: `fd` is outside the open-file table or does not name an open
///     file descriptor.
///   - `ENOENT`: the open file table entry has no inode attached.
///   - `EROFS`: the target inode is on a read-only filesystem.
///   - `EPERM`: the caller is neither superuser nor the inode owner requesting
///     the existing owner together with either the existing group or a group
///     the caller belongs to.
///
/// Filesystem-specific `notify_change` hooks may return additional errors.
/// The verified Linux 1.0 cases are `EPERM` from the MSDOS ownership policy
/// and NFS transport/server errors such as `EIO`, `ESTALE`, `EDQUOT`, and
/// other negative errnos forwarded by `nfs_proc_setattr()`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/fchown.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n33)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n33)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n304)
///
/// # Historical References
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n104)
pub fn fchown16(fd: UnsignedInt, user: OldUidT, group: OldGidT) -> Int {
    // SAFETY: `fchown16` takes only integer arguments and has no caller-side
    // memory-safety preconditions.
    unsafe {
        syscall3(Sysno::Fchown, fd as isize, user as isize, group as isize)
            as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::AsRawFd as _,
        os::unix::fs::MetadataExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OldGidT, OldUidT, UnsignedInt};

    use crate::arch::current::Sysno;

    use super::fchown16;

    fn create_temp_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_sys_fchown_test_{now}"));

        path
    }

    #[test]
    fn test_fchown16_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 95;
        #[cfg(target_arch = "aarch64")]
        let expected = 55;

        assert_eq!(Sysno::Fchown as isize, expected);
    }

    #[test]
    fn test_fchown16_current_ids() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let metadata = file.metadata().unwrap();
        let uid = metadata.uid();
        let gid = metadata.gid();

        if u64::from(uid) > u64::from(OldUidT::MAX)
            || u64::from(gid) > u64::from(OldGidT::MAX)
        {
            fs::remove_file(&path).unwrap();
            return;
        }

        let result = fchown16(
            file.as_raw_fd() as UnsignedInt,
            uid as OldUidT,
            gid as OldGidT,
        );
        assert_eq!(result, 0, "fchown16 failed: {result}");

        let updated = fs::metadata(&path).unwrap();
        assert_eq!(updated.uid(), uid);
        assert_eq!(updated.gid(), gid);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchown16_no_change_sentinels() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let metadata = file.metadata().unwrap();

        let result = fchown16(
            file.as_raw_fd() as UnsignedInt,
            OldUidT::MAX,
            OldGidT::MAX,
        );
        assert_eq!(result, 0, "fchown16 failed: {result}");

        let updated = fs::metadata(&path).unwrap();
        assert_eq!(updated.uid(), metadata.uid());
        assert_eq!(updated.gid(), metadata.gid());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchown16_preserve_uid_with_current_gid() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let metadata = file.metadata().unwrap();
        let gid = metadata.gid();

        if u64::from(gid) > u64::from(OldGidT::MAX) {
            fs::remove_file(&path).unwrap();
            return;
        }

        let result = fchown16(
            file.as_raw_fd() as UnsignedInt,
            OldUidT::MAX,
            gid as OldGidT,
        );
        assert_eq!(result, 0, "fchown16 failed: {result}");

        let updated = fs::metadata(&path).unwrap();
        assert_eq!(updated.uid(), metadata.uid());
        assert_eq!(updated.gid(), gid);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchown16_invalid_fd() {
        let result = fchown16(UnsignedInt::MAX, OldUidT::MAX, OldGidT::MAX);

        assert_eq!(result, -9, "expected EBADF from invalid fd, got {result}");
    }
}
