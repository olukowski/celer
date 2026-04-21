use celer_system_linux_ctypes::{Char, Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Change the owner and/or group of a file without following a symlink.
///
/// # Safety
/// - `filename` must point to a NUL-terminated string that is readable for the
///   duration of the syscall.
/// - Any irreversible side effects of changing the file ownership are intended.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: the name and kernel entry point were split from the
///   historical `chown` naming over time.
/// - Availability: always present on supported Linux kernels
///
/// # Naming Notes
/// - Linux 1.0 exposed this behavior as `sys_chown`.
/// - Modern kernels expose the no-follow behavior as `lchown`.
/// - The plain `chown` name now refers to the follow-symlinks variant.
///
/// # Behavior
/// - The syscall delegates to `do_fchownat(AT_FDCWD, filename, user, group,
///   AT_SYMLINK_NOFOLLOW)`.
/// - The kernel resolves the path without following the final symlink, then
///   applies the ownership change to the referenced path component.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/lchown.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n842)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n838)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n250)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n104)
pub unsafe fn lchown(
    filename: *const Char,
    user: UnsignedInt,
    group: UnsignedInt,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::Lchown,
            filename.addr() as isize,
            user as isize,
            group as isize,
        ) as Int
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        os::unix::fs::MetadataExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, UnsignedInt};

    use super::lchown;

    fn create_temp_file(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));
        fs::write(&path, b"test").unwrap();

        path
    }

    #[test]
    fn test_lchown_group() {
        let path = create_temp_file("test_lchown_group");
        let gid: UnsignedInt =
            fs::metadata(&path).unwrap().gid() as UnsignedInt;

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        // SAFETY: `path_bytes` is NUL-terminated and readable for the
        // duration of the syscall.
        let result = unsafe {
            lchown(path_bytes.as_ptr().cast::<Char>(), !0 as UnsignedInt, gid)
        };
        assert_eq!(result, 0, "lchown failed: {result}");

        let meta = fs::metadata(&path).unwrap();
        assert_eq!(meta.gid(), gid as u32);

        fs::remove_file(&path).unwrap();
    }
}
