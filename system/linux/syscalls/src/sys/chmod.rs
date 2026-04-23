use celer_system_linux_ctypes::{Char, Int, UModeT};

use crate::arch::current::{Sysno, syscall2};

/// Change the mode bits of a file named by `pathname`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Resolves `pathname` to a file and updates its mode bits.
/// - On success, returns `0`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/chmod.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n716)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n719)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n276)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n105)
pub fn chmod(pathname: *const Char, mode: UModeT) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe {
        syscall2(Sysno::Chmod, pathname.addr() as isize, mode as isize) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::unix::fs::PermissionsExt as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::chmod;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_chmod_{now}"));

        path
    }

    #[test]
    fn test_chmod() {
        let path = create_temp_path();
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let result = chmod(path_bytes.as_ptr().cast::<Char>(), 0o600);
        assert_eq!(result, 0, "chmod failed: {result}");

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        fs::remove_file(&path).unwrap();
    }
}
