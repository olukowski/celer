use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Load a shared library using the historical `uselib` ABI.
///
/// Linux 1.0 implements `uselib` as a real shared-library loader entrypoint.
/// In the checked current stable, LTS, and mainline trees, the syscall still
/// has declarations and syscall-table entries, but this wrapper does not rely
/// on any unverified current-kernel implementation details beyond those
/// sources.
///
/// # Kernel Support
/// - Introduced: Linux 0.12
/// - Behavior changes: Linux 1.0 opens `library` read-only and iterates the
///   registered `load_shlib` handlers until one succeeds or returns an error
///   other than `ENOEXEC`
/// - Availability: Linux 1.0 provides a real implementation; the checked
///   current stable, LTS, and mainline trees still declare the syscall, but
///   this wrapper does not assume a specific current-kernel implementation
///
/// # Required Privileges
/// - None beyond whatever file access the kernel's pathname resolution and
///   loader path require.
///
/// # Behavior
/// - Linux 1.0 opens `library` with `sys_open(library, 0, 0)`.
/// - If the open succeeds and the resulting file has a readable file
///   operation, Linux 1.0 walks the registered binary formats in order and
///   calls each format's `load_shlib` hook.
/// - The Linux 1.0 loader loop continues while the previous loader returned
///   `-ENOEXEC`.
/// - Linux 1.0 returns `-ENOEXEC` if the file opens but no registered loader
///   accepts it.
///
/// # Errors
/// - `EFAULT`: `library` is null or points outside the task address space.
/// - `ENAMETOOLONG`: the copied pathname does not fit in one page.
/// - `ENOENT`: the pathname is empty or a path component cannot be resolved.
/// - `ENOMEM`: pathname-buffer allocation failed.
/// - `ENOTDIR`: a non-final path component is not a directory.
/// - `EACCES`: pathname resolution denied access, or a recognized Linux 1.0
///   shared-library loader rejected a short header read.
/// - `EMFILE`: the process had no free file descriptor for the internal open.
/// - `ENFILE`: the system had no free file table entry for the internal open.
/// - `ENOEXEC`: Linux 1.0 opened the file but no registered loader accepted
///   its format.
///
/// The Linux 1.0 loader helpers may surface additional errors after `sys_open`
/// succeeds. This wrapper documents the reachable errno values verified along
/// the Linux 1.0 entry path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/uselib.2.html)
/// - Stable declaration:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/linux/syscalls.h?h=v7.0#n1114)
/// - Stable generic fallback:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys_ni.c?h=v7.0#n329)
/// - LTS declaration:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/linux/syscalls.h?h=v6.18.18#n1110)
/// - LTS generic fallback:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys_ni.c?h=v6.18.18#n329)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/exec.c?h=1.0#n238)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/exec.c?h=0.12#n42)
pub fn uselib(library: *const Char) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe { syscall1(Sysno::Uselib, library.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use celer_system_linux_ctypes::Int;

    use crate::arch::current::Sysno;

    use super::uselib;

    #[test]
    fn test_uselib_syscall_number() {
        assert_eq!(Sysno::Uselib as isize, 86);
    }

    #[test]
    fn test_uselib_null_pointer_is_rejected_or_unimplemented() {
        let ret = uselib(core::ptr::null());
        let expected = [-14, -38];

        assert!(
            expected.contains(&ret),
            "expected EFAULT or ENOSYS from uselib(null), got {ret}",
        );
    }

    #[test]
    fn test_uselib_missing_library_reports_path_error_or_unimplemented() {
        let path =
            CString::new("/definitely/not/a/real/celer-uselib-library.so")
                .unwrap();
        let ret = uselib(path.as_ptr().cast());
        let expected: [Int; 3] = [-2, -8, -38];

        assert!(
            expected.contains(&ret),
            "expected ENOENT, ENOEXEC, or ENOSYS from uselib(missing path), got {ret}",
        );
    }
}
