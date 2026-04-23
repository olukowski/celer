use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Change the calling process's root directory.
///
/// # Kernel Support
/// - Available in Linux 1.0
/// - Behavior changes: current kernels add path-permission, capability, and
///   security-hook checks before changing the root
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - The caller must be privileged enough for the kernel to permit the root
///   change.
///
/// # Behavior
/// - Resolves `name` to a directory.
/// - On success, updates the calling process's root directory to the resolved
///   path.
/// - This syscall changes the caller's root state; tasks sharing the same
///   `fs_struct` can observe the change.
///
/// # Errors
/// - `EFAULT`: `name` is null or not readable for a full NUL-terminated path.
/// - `ENOENT`: the path does not exist.
/// - `ENOTDIR`: the resolved object is not a directory.
/// - `EPERM`: the caller is not permitted to change root.
/// - Other pathname-lookup and permission errors may be returned by the VFS
///   path resolution helpers.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/chroot.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n588)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n588)
/// - First stable implementation: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n232)
pub fn chroot(name: *const Char) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe { syscall1(Sysno::Chroot, name.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::chroot;

    #[test]
    fn test_chroot_invalid_path() {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_chroot_missing_{now}"));

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let result = chroot(path_bytes.as_ptr().cast::<Char>());
        assert_eq!(result, -2, "chroot should fail with ENOENT: {result}");
    }

    #[test]
    fn test_chroot_not_directory() {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_chroot_file_{now}"));
        File::create(&path).unwrap();

        let mut path_bytes = path.as_os_str().as_encoded_bytes().to_vec();
        path_bytes.push(0);

        let result = chroot(path_bytes.as_ptr().cast::<Char>());
        assert_eq!(result, -20, "chroot should fail with ENOTDIR: {result}");

        fs::remove_file(&path).unwrap();
    }
}
