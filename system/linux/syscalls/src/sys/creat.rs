use celer_system_linux_ctypes::{Char, Long, UModeT};

use crate::arch::current::{Sysno, syscall2};

/// Create or truncate a file named by `pathname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel opens `pathname` with `O_CREAT | O_WRONLY | O_TRUNC`.
/// - If the relevant large-file condition is met, the kernel adds
///   `O_LARGEFILE` before opening the file.
///
/// # Errors
/// - The syscall returns the same errors as the underlying open path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/creat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n1508)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n1524)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n437)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n138)
pub fn creat(pathname: *const Char, mode: UModeT) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe { syscall2(Sysno::Creat, pathname.addr() as isize, mode as isize) })
        as Long
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::fd::{AsRawFd as _, FromRawFd as _},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, UModeT};

    use super::creat;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_creat_{now}"));

        path
    }

    #[test]
    fn test_creat() {
        let path = create_temp_path();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let fd = creat(path_bytes.as_ptr().cast::<Char>(), 0o644 as UModeT);

        assert!(fd >= 0, "creat failed: {}", fd);

        let raw_fd = fd as i32;
        let opened = unsafe { File::from_raw_fd(raw_fd) };
        assert_eq!(opened.as_raw_fd(), raw_fd);

        fs::remove_file(&path).unwrap();
    }
}
