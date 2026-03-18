use celer_system_linux_ctypes::{Char, Long, SizeT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Write up to `count` bytes from the buffer starting at `buf`
/// to the file referred to by the file descriptor `fd`.
///
/// # Safety
/// - `buf` must be readable for `count` bytes (see [`core::ptr::read`]).
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
/// - On files that support seeking, the write starts at the current file
///   offset and the offset is incremented by the number of bytes written.
/// - If `count` is 0 and `fd` is a regular file, [`write`] returns 0 without
///   reading any data, but may still detect errors.
/// - On success, returns the number of bytes written, which lies in the range
///   `0..=count`. A partial write is *not* considered an error, it can occur
///   naturally (i.e., the write was interrupted by a signal).
///
/// # Errors
/// - `EAGAIN`: The file descriptor does not refer to a socket,
///   is marked nonblocking, and the write would block.
/// - `EAGAIN` or `EWOULDBLOCK`: The file descriptor refers to a socket that
///   is marked nonblocking, and the write would block.
/// - `EBADF`: The file descriptor is invalid, or not open for writing.
/// - `EDESTADDRREQ`: The file descriptor refers to a socket and the socket is
///   not connected.
/// - `EDQUOT`: The file descriptor refers to a file and the user's quota of
///   disk blocks or inodes has been exhausted.
/// - `EFAULT`: The `buf` pointer is outside the process's accessible address
///   space.
/// - `EFBIG`: The write would exceed the maximum file size or the process's
///   file size limit, or write at a position past the allowed offset.
/// - `EINTR`: The write was interrupted by a signal (before any data was
///   written).
/// - `EINVAL`: The file descriptor is unsuitable for writing or the file was
///   opened with the `O_DIRECT` flag and `buf`, `count` or the file offset are
///   not suitably aligned.
/// - `EIO`: An I/O error occurred during the write.
/// - `ENOSPC`: The device containing the file does not have enough free space.
/// - `EPERM`: The operation was prevented by a file seal.
/// - `EPIPE`: The file descriptor refers to a pipe or socket whose read end is
///   closed. A `SIGPIPE` signal is also generated.
///
/// Other errors may also occur, depending on the type of object being written.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/write.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v6.19#n746)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n746)
/// - First stable [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n90)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.10#n83)
pub unsafe fn write(fd: UnsignedInt, buf: *const Char, count: SizeT) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall3(
            Sysno::Write,
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
        io::{Read, Seek as _},
        os::fd::AsRawFd as _,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{SizeT, UnsignedInt};

    use super::write;

    fn create_temp_path() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_write_{now}"));

        path
    }

    #[test]
    fn test_write() {
        let path = create_temp_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .unwrap();

        let msg = b"Hello, World!";

        let mut to_write: &[u8] = msg;
        while !to_write.is_empty() {
            // SAFETY: `to_write.as_ptr()` is readable for `to_write.len()`
            // bytes
            let result = unsafe {
                write(
                    file.as_raw_fd() as UnsignedInt,
                    to_write.as_ptr().cast(),
                    to_write.len() as SizeT,
                )
            };

            assert!(result >= 0, "write failed: {}", result);

            let written = result as usize;
            to_write = &to_write[written..];
        }

        file.rewind().unwrap();

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, msg);

        fs::remove_file(&path).unwrap()
    }
}
