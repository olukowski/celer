use celer_system_linux_ctypes::Int;

use crate::arch::current::{Sysno, syscall1};

/// Close an open file descriptor.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - The kernel first looks up the file descriptor and returns `EBADF` if it is
///   not open.
/// - On success, the file descriptor is removed from the caller's file table and
///   the kernel flushes the underlying file before releasing it.
/// - If the flush path hits a restart request, the kernel converts it to `EINTR`
///   before returning.
///
/// # Errors
/// - `EBADF`: `fd` does not refer to an open file descriptor.
/// - `EINTR`: the close path was interrupted and the kernel converted a restart
///   request into an interrupt error.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/close.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/open.c?h=v6.19#n1558)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/open.c?h=v6.18.18#n1574)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n465)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=0.10#n192)
pub fn close(fd: Int) -> Int {
    // SAFETY: close is always safe to call.
    unsafe { syscall1(Sysno::Close, fd as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
    };

    use celer_system_linux_ctypes::Int;

    use super::close;

    #[test]
    fn test_close() {
        let path = std::env::temp_dir().join("celer_sys_close_test");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        let result = close(fd as Int);
        assert_eq!(result, 0);

        fs::remove_file(&path).unwrap();
    }
}
