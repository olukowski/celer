use celer_system_linux_ctypes::{Char, Int, Long, UModeT};

use crate::arch::current::{Sysno, syscall3};

/// Open a file named by `filename` with the given `flags` and `mode`.
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
/// - On success, returns a new file descriptor referring to the opened file.
/// - The returned descriptor is the lowest-numbered unused descriptor in the
///   calling process.
/// - If `flags` includes `O_CREAT`, the `mode` argument controls the file mode
///   bits used when creating the file.
///
/// # Errors
/// - `EINVAL`: Invalid flag combinations are rejected by `build_open_flags()`.
///
/// Lower-level VFS and path-resolution helpers may return additional errno
/// values; see the referenced kernel source for the full chain.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/open.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n1374)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n1374)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n424)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n138)
pub fn open(filename: *const Char, flags: Int, mode: UModeT) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall3(
            Sysno::Open,
            filename.addr() as isize,
            flags as isize,
            mode as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::fd::{AsRawFd as _, FromRawFd as _},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, Int, UModeT};

    use super::open;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_open_{now}"));

        path
    }

    #[test]
    fn test_open() {
        let path = create_temp_path();
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let fd =
            open(path_bytes.as_ptr().cast::<Char>(), 0 as Int, 0 as UModeT);

        assert!(fd >= 0, "open failed: {}", fd);

        let raw_fd = fd as i32;
        let opened = unsafe { File::from_raw_fd(raw_fd) };
        assert_eq!(opened.as_raw_fd(), raw_fd);

        fs::remove_file(&path).unwrap();
    }
}
