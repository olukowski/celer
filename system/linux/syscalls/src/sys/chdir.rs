use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Change the calling process's current working directory.
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
/// - Resolves `filename` as a directory path.
/// - Checks execute permission on the resolved directory.
/// - On success, updates the caller's current working directory.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/chdir.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n552)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n550)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n192)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n75)
pub fn chdir(filename: *const Char) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe { syscall1(Sysno::Chdir, filename.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use crate::sys::test_support::process_global_state_guard;

    use super::chdir;

    fn create_temp_dir() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_chdir_{now}"));
        fs::create_dir(&path).unwrap();

        path
    }

    #[test]
    fn test_chdir() {
        let _guard = process_global_state_guard();
        let original_dir = env::current_dir().unwrap();
        let path = create_temp_dir();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let result = chdir(path_bytes.as_ptr().cast::<Char>());
        assert_eq!(result, 0, "chdir failed: {result}");

        assert_eq!(env::current_dir().unwrap(), path);

        let mut original_bytes =
            original_dir.as_os_str().as_encoded_bytes().to_vec();
        original_bytes.push(0);

        let restore = chdir(original_bytes.as_ptr().cast::<Char>());
        assert_eq!(restore, 0, "restoring cwd failed: {restore}");

        fs::remove_dir(&path).unwrap();
    }
}
