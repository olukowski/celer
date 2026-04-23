use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall2};

/// Create a hard link from `newname` to `oldname`.
///
/// # Safety
/// - Both pathname pointers must be valid to read NUL-terminated strings for
///   the duration of the syscall.
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
/// - The syscall delegates to `do_linkat(AT_FDCWD, getname(oldname),
///   AT_FDCWD, getname(newname), 0)`.
/// - On success, `newname` becomes another directory entry for the same
///   underlying inode as `oldname`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/link.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n5757)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n5757)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n656)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.10#n712)
pub unsafe fn link(oldname: *const Char, newname: *const Char) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall2(
            Sysno::Link,
            oldname.addr() as isize,
            newname.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::unix::fs::MetadataExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::link;

    fn create_temp_path(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("{prefix}_{now}"));

        path
    }

    #[test]
    fn test_link() {
        let old_path = create_temp_path("test_link_old");
        let new_path = create_temp_path("test_link_new");

        File::create(&old_path).unwrap();

        let mut old_bytes = old_path.as_os_str().as_encoded_bytes().to_vec();
        old_bytes.push(0);
        let mut new_bytes = new_path.as_os_str().as_encoded_bytes().to_vec();
        new_bytes.push(0);

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let ret = unsafe {
            link(
                old_bytes.as_ptr().cast::<Char>(),
                new_bytes.as_ptr().cast::<Char>(),
            )
        };

        assert_eq!(ret, 0, "link failed: {ret}");

        let old_meta = fs::metadata(&old_path).unwrap();
        let new_meta = fs::metadata(&new_path).unwrap();

        assert_eq!(old_meta.ino(), new_meta.ino());
        assert_eq!(old_meta.dev(), new_meta.dev());

        fs::remove_file(&old_path).unwrap();
        fs::remove_file(&new_path).unwrap();
    }
}
