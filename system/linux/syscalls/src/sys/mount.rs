use celer_system_linux_ctypes::{Char, Long, UnsignedLong, Void};

use crate::arch::current::{Sysno, syscall5};

/// Mount or remount a filesystem.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Errors
/// - `EFAULT`: one of the user pointers cannot be read while copying the mount
///   string or mount options.
/// - `ENOMEM`: the kernel cannot allocate memory for copied mount arguments.
/// - `EPERM`: the caller lacks permission to perform the mount.
/// - `EINVAL`: the kernel rejects the requested flags or mount configuration.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/mount.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/namespace.c?h=v6.19#n2093)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/namespace.c?h=v6.18.18#n2093)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=1.0#n427)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/super.c?h=0.10#n199)
pub fn mount(
    source: *const Char,
    target: *const Char,
    filesystemtype: *const Char,
    mountflags: UnsignedLong,
    data: *const Void,
) -> Long {
    // SAFETY: this wrapper forwards the raw user pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall5(
            Sysno::Mount,
            source.addr() as isize,
            target.addr() as isize,
            filesystemtype.addr() as isize,
            mountflags as isize,
            data.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::UnsignedLong;

    use super::mount;

    #[test]
    fn test_mount_invalid_parameters() {
        let ret = mount(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            0 as UnsignedLong,
            core::ptr::null(),
        );

        assert!(ret < 0, "mount unexpectedly succeeded: {}", ret);
    }
}
