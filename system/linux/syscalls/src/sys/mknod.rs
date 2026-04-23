use celer_system_linux_ctypes::{Char, Long, UModeT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Create the node named by `pathname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Regular-file creation is unprivileged; other node types may be subject to
///   additional kernel policy.
///
/// # Behavior
/// - The syscall delegates to `do_mknodat(AT_FDCWD, getname(filename), mode,
///   dev)`.
/// - `mode` controls the node type; the kernel accepts regular files, device
///   nodes, FIFOs, and sockets, and rejects directories.
///
/// # Errors
/// - The kernel may return pathname-resolution, permission, and VFS errors
///   depending on the target path, filesystem, and node type.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/mknod.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namei.c?h=v6.19#n5083)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namei.c?h=v6.18.18#n5083)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=1.0#n416)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/namei.c?h=0.10#n408)
pub fn mknod(pathname: *const Char, mode: UModeT, dev: UnsignedInt) -> Long {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall3(
            Sysno::Mknod,
            pathname.addr() as isize,
            mode as isize,
            dev as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, UModeT, UnsignedInt};

    use super::mknod;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_mknod_{now}"));

        path
    }

    #[test]
    fn test_mknod_regular_file() {
        let path = create_temp_path();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let rc = mknod(
            path_bytes.as_ptr().cast::<Char>(),
            (0o100000 | 0o600) as UModeT,
            0 as UnsignedInt,
        );

        assert_eq!(rc, 0, "mknod failed: {rc}");
        assert!(path.exists(), "mknod did not create the file");
        assert!(fs::metadata(&path).unwrap().is_file());

        fs::remove_file(&path).unwrap();
    }
}
