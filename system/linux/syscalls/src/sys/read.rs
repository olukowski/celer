use celer_system_linux_ctypes::{Char, Long, SizeT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Attempt to read up to `count` bytes from file descriptor `fd`
/// into the buffer starting at `buf`.
///
/// # Safety
/// - `buf` must be writable for `count` bytes (see [`core::ptr::write`]).
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On files that support seeking, the read starts at the current file offset
///   and the offset is incremented by the number of bytes read.
///   If the the file offset is at or beyond the end of the file,
///   [`read`] returns 0 to indicate end of file.
/// - If `count` is 0, [`read`] returns 0 without reading any data,
///   but may still detect errors.
/// - On success, returns the number of bytes read, or `0` to indicate
///   end of file. The number of bytes read lies in the range `1..=count`.
///   A partial read is *not* considered an error, it can occur naturally
///   (i.e., the end of the file is reached before `count` bytes are read).
///
/// # Errors
/// - `EAGAIN`: The file descriptor does not refer to a socket,
///   is marked nonblocking, and the read would block.
/// - `EAGAIN` or `EWOULDBLOCK`: The file descriptor refers to a socket that
///   is marked nonblocking, and the read would block.
/// - `EBADF`: The file descriptor is invalid, or not open for reading.
/// - `EFAULT`: The `buf` pointer is outside the process's accessible address
///   space.
/// - `EINTR`: The read was interrupted by a signal (before any data was read).
/// - `EINVAL`: The file descriptor is unsuitable for reading or the file was
///   opened with the `O_DIRECT` flag and `buf`, `count` or the file offset are
///   not suitably aligned.
/// - `EINVAL`: `fd` was created via a call to `timerfd_create` and the wrong
///   size buffer was passed to [`read`].
/// - `EIO`: An I/O error occurred while reading from the file.
/// - `EISDIR`: The file descriptor refers to a directory, not a regular file.
///
/// Other errors may also occur, depending on the type of object being read.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/read.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v6.19#n722)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n722)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n70)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.10#n55)
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
mod tests {
    use std::{
        env,
        fs::{self, OpenOptions},
        io::Write as _,
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

        let mut buf = [0u8; 32];

        while !contents_to_check.is_empty() {
            let n = unsafe {
                read(
                    file.as_raw_fd() as UnsignedInt,
                    buf.as_mut_ptr().cast(),
                    contents_to_check.len() as SizeT,
                )
            } as usize;

            assert_ne!(n, 0); // we should NOT reach EOF

            // just retry on error
            if n > 0 {
                assert_eq!(&buf[..n], &contents_to_check[..n]);

                contents_to_check = &contents_to_check[n..];
            }
        }

        fs::remove_file(&path).unwrap()
    }
}
