use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall1};

/// Remove a directory entry named by `pathname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, removes the named directory entry.
/// - The kernel implementation forwards the call through `filename_unlinkat`
///   with `AT_FDCWD`, so the path is interpreted relative to the calling
///   process's current working directory.
///
/// # Errors
/// - The kernel may return pathname resolution and VFS errors such as `ENOENT`
///   or `EPERM`, depending on the target path and filesystem.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n5600)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n5600)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n553)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.10#n647)
pub fn unlink(pathname: *const Char) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe { syscall1(Sysno::Unlink, pathname.addr() as isize) }) as Long
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::File,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::unlink;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_unlink_{now}"));

        path
    }

    #[test]
    fn test_unlink() {
        let path = create_temp_path();
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = unlink(path_bytes.as_ptr().cast::<Char>());

        assert_eq!(rc, 0, "unlink failed: {}", rc);
        assert!(!path.exists(), "unlink did not remove the file");
    }
}
