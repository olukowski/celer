use celer_system_linux_ctypes::{Char, Long, SizeT, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Write up to `count` bytes from the buffer starting at `buf`
/// to the file referred to by the file descriptor `fd`.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes:
///   - Linux 1.0 required `file->f_op->write`; current kernels also accept
///     `file->f_op->write_iter`.
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
///   before the file's write implementation runs.
/// - On current kernels, seekable files use a temporary copy of the current
///   file position and write the updated offset back on success.
/// - On success, returns the number of bytes written.
/// - Additional object-specific behavior comes from the target file's
///   `->write` or `->write_iter` implementation.
///
/// # Errors
/// - `EBADF`: `fd` is not open, or is not open for writing.
/// - `EFAULT`: `buf` is not accessible for `count` bytes.
/// - `EINVAL`: Linux 1.0 rejects files without `->write`; current kernels
///   also reject files that have neither `->write` nor `->write_iter`, and
///   `rw_verify_area()` can reject invalid write requests with `EINVAL`.
/// - Other reachable errors are returned by `rw_verify_area()` or by the
///   target file's write implementation.
///
/// # References
/// - Stable entry:
///   [v7.0 write](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v7.0#n748)
/// - Stable helper:
///   [v7.0 vfs_write](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/read_write.c?h=v7.0#n668)
/// - LTS entry:
///   [v6.18.18 write](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n746)
/// - LTS helper:
///   [v6.18.18 vfs_write](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/read_write.c?h=v6.18.18#n666)
/// - First stable:
///   [Linux 1.0 sys_write](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n90)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.10 sys_write](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.10#n83)
pub fn write(fd: UnsignedInt, buf: *const Char, count: SizeT) -> Long {
    // SAFETY: this wrapper forwards the raw user pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
            let result = write(
                file.as_raw_fd() as UnsignedInt,
                to_write.as_ptr().cast(),
                to_write.len() as SizeT,
            );

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
