use celer_system_linux_ctypes::{Char, Long, SizeT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Attempt to read up to `count` bytes from file descriptor `fd`
/// into the buffer starting at `buf`.
///
/// # Safety
/// - `buf` must be writable for `count` bytes (see [`core::ptr::write`]).
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes:
///   - Linux 1.0 required `file->f_op->read`; current kernels also accept
///     `file->f_op->read_iter`.
///   - Linux 1.0 returned `0` immediately when `count == 0`; current kernels
///     still validate the descriptor, access mode, and user buffer, and then
///     clamp `count` to `MAX_RW_COUNT` before dispatch.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On current kernels, `count > MAX_RW_COUNT` is reduced to `MAX_RW_COUNT`
///   before the file's read implementation runs.
/// - On current kernels, seekable files use a temporary copy of the current
///   file position and write the updated offset back on success.
/// - On success, returns the number of bytes read.
/// - Additional object-specific behavior comes from the target file's
///   `->read` or `->read_iter` implementation.
///
/// # Errors
/// - `EBADF`: `fd` is not open, or is not open for reading.
/// - `EFAULT`: `buf` is not accessible for `count` bytes.
/// - `EINVAL`: Linux 1.0 rejects files without `->read`; current kernels also
///   reject files that have neither `->read` nor `->read_iter`, and
///   `rw_verify_area()` can reject invalid read requests with `EINVAL`.
/// - Other reachable errors are returned by `rw_verify_area()` or by the
///   target file's read implementation.
///
/// # References
/// - Stable entry:
///   [v7.0 read](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v7.0#n724)
/// - Stable helper:
///   [v7.0 vfs_read](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v7.0#n554)
/// - LTS entry:
///   [v6.18.18 read](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n722)
/// - LTS helper:
///   [v6.18.18 vfs_read](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n552)
/// - First stable:
///   [Linux 1.0 sys_read](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n70)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.10 sys_read](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.10#n55)
pub unsafe fn read(fd: UnsignedInt, buf: *mut Char, count: SizeT) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(
            Sysno::Read,
            fd as isize,
            buf.addr() as isize,
            count as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, OpenOptions},
        io::{Seek as _, Write as _},
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{SizeT, UnsignedInt};

    use super::read;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_read_{now}"));

        path
    }

    #[test]
    fn test_read() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();

        let mut contents_to_check: &[u8] = b"Hello, World!";
        file.write_all(contents_to_check).unwrap();
        file.rewind().unwrap();

        let mut buf = [0u8; 32];

        while !contents_to_check.is_empty() {
            // SAFETY: `buf.as_mut_ptr()` is writable for `32` bytes,
            // which is more than `contents_to_check.len()`
            let n = unsafe {
                read(
                    file.as_raw_fd() as UnsignedInt,
                    buf.as_mut_ptr().cast(),
                    contents_to_check.len() as SizeT,
                )
            } as usize;

            assert!(n > 0); // we should NOT reach EOF, or get an error

            assert_eq!(&buf[..n], &contents_to_check[..n]);

            contents_to_check = &contents_to_check[n..];
        }

        fs::remove_file(&path).unwrap()
    }
}
