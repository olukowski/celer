use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall1};

/// Create a pipe and write the read and write file descriptors to `fildes`.
///
/// # Safety
/// - `fildes` must be valid to write two `Int` values for the duration of the
///   syscall.
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
/// - On success, writes the read end to `fildes[0]` and the write end to
///   `fildes[1]`.
/// - The legacy `pipe` entry always creates an ordinary pipe with no flags.
///
/// # Errors
/// - `EFAULT`: `fildes` is not writable for two `Int` values.
/// - `EMFILE`: the caller cannot obtain two more file descriptor slots.
/// - `ENFILE`: pipe or file-object allocation failed in the kernel path.
/// - `ENOMEM`: kernel allocation of pipe metadata or file objects failed.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/pipe.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/pipe.c?h=v6.19#n1059)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/pipe.c?h=v6.18.18#n1059)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/pipe.c?h=0.10#n71)
pub unsafe fn pipe(fildes: *mut Int) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Pipe, fildes.addr() as isize) as Int }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Int;

    use crate::sys::close;

    use super::pipe;

    #[test]
    fn test_pipe() {
        let mut fds = [0 as Int; 2];

        // SAFETY: `fds` is writable for two `Int` values.
        let rc = unsafe { pipe(fds.as_mut_ptr()) };
        assert_eq!(rc, 0, "pipe failed: {rc}");
        assert_ne!(fds[0], fds[1]);

        assert_eq!(close(fds[0]), 0);
        assert_eq!(close(fds[1]), 0);
    }

    #[test]
    fn test_pipe_null() {
        // SAFETY: passing a null pointer is intentionally invalid here.
        let rc = unsafe { pipe(core::ptr::null_mut()) };

        assert_eq!(rc, -14, "expected EFAULT from null pipe buffer, got {rc}");
    }
}
