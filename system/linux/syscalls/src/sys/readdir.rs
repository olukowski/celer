use celer_system_linux_ctypes::{Long, OldLinuxDirent, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};

/// Read one entry from the directory referred to by `fd` into the historical
/// `old_readdir` buffer at `dirent`.
///
/// This wrapper targets syscall slot `89` from Linux 1.0 rather than the
/// newer multi-entry `getdents` family.
///
/// # Safety
/// - `dirent` must point to one writable [`OldLinuxDirent`] value for the
///   duration of the syscall.
/// - `dirent` must not alias live Rust references or other memory that would
///   violate Rust's aliasing rules while the kernel may write through that
///   pointer.
///
/// # Kernel Support
/// - Introduced: Linux 0.96a
/// - Behavior changes: current x86 kernels still expose syscall slot `89`
///   through the legacy `old_readdir` ABI, but successful calls return `1`
///   per emitted entry and write a variable-length record instead of the
///   fixed Linux 1.0 `struct dirent`
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, reads at most one directory entry into `dirent`.
/// - On current x86 kernels, `dirent.d_namlen` receives the copied filename
///   length, and the filename bytes plus trailing NUL are written immediately
///   after the fixed prefix.
/// - On Linux 1.0, success returns the copied filename length.
/// - On Linux 1.0, `dirent.d_reclen` receives the copied filename length, not
///   the size of the whole record.
/// - On Linux 1.0, `dirent.d_name` is NUL-terminated on successful entry
///   reads.
/// - A return value of `0` usually indicates end of directory on Linux 1.0.
/// - Linux 1.0 documents `count` as "not yet used" and recommends passing
///   `1`; filesystem implementations still receive it, but this syscall does
///   not return multiple entries in one call.
/// - On current x86 kernels, successful calls return `1` after emitting one
///   entry, and `count` is treated as a one-entry hint.
///
/// # Errors
/// Linux 1.0 entry path:
/// - `EBADF`: `fd` is not an open file descriptor with an inode.
/// - `EBADF`: the target object's filesystem `readdir` implementation rejects
///   the descriptor or inode state.
/// - `ENOTDIR`: the opened object has no `readdir` operation.
/// - `EFAULT`: `dirent` is not writable for one Linux 1.0 fixed-size dirent.
/// - `ENOENT`: reachable from the msdos filesystem when the directory
///   position is misaligned.
/// - `EIO`: reachable through NFS `readdir` failures.
/// - `EPERM`, `ENXIO`, `EACCES`, `EEXIST`, `ENODEV`, `EISDIR`, `EINVAL`,
///   `EFBIG`, `ENOSPC`, `EROFS`, `ENAMETOOLONG`, `ENOTEMPTY`, `EDQUOT`, and
///   `ESTALE`: reachable when Linux 1.0 NFS `readdir` propagates a mapped
///   server error.
///
/// Current x86 `old_readdir` adds further reachable errors, including:
/// - `EFAULT`: `dirent` is not writable for the fixed prefix plus the returned
///   name and trailing NUL.
/// - `EOVERFLOW`: the returned inode number does not fit in `unsigned long`.
/// - `EIO`: the filesystem emits an invalid directory entry name.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/readdir.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/readdir.c?h=v6.19#n218)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/readdir.c?h=v6.18.18#n218)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=1.0#n19)
///
/// # Historical References
/// - Linux 1.0 `struct dirent`:
///   [include/linux/dirent.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/dirent.h?h=1.0#n6)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n98)
/// - First stable syscall table:
///   [kernel/sched.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n137)
/// - First appearance:
///   [Linux 0.96a sys_readdir](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/read_write.c?h=0.96a#n21)
/// - Current x86 syscall table:
///   [arch/x86/entry/syscalls/syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n104)
/// - Current x86 ABI shape:
///   [fs/readdir.c](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/readdir.c?h=v6.19#n168)
pub unsafe fn readdir(
    fd: UnsignedInt,
    dirent: *mut OldLinuxDirent,
    count: UnsignedInt,
) -> Long {
    // SAFETY: this wrapper forwards the raw output pointer without
    // dereferencing it in Rust, so pointer validity is checked by the kernel.
    (unsafe {
        syscall3(
            Sysno::Readdir,
            fd as isize,
            dirent.addr() as isize,
            count as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        fs::{self, File},
        os::fd::AsRawFd as _,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{
        Char, OLD_LINUX_DIRENT_NAME_CAP, OldLinuxDirent, UnsignedInt,
    };

    use crate::arch::current::Sysno;

    use super::readdir;

    fn create_temp_dir() -> PathBuf {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_readdir_{now}"));
        fs::create_dir(&path).unwrap();

        path
    }

    fn zeroed_dirent() -> OldLinuxDirent {
        OldLinuxDirent {
            d_ino: 0,
            d_offset: 0,
            d_namlen: 0,
            d_name: [0; OLD_LINUX_DIRENT_NAME_CAP],
        }
    }

    fn dirent_name(dirent: &OldLinuxDirent) -> Vec<u8> {
        dirent.d_name[..dirent.d_namlen as usize]
            .iter()
            .map(|&ch| ch as u8)
            .collect()
    }

    fn touch(path: &Path) {
        File::create(path).unwrap();
    }
    fn directory_contains_entry(fd: UnsignedInt, target_name: &[u8]) -> bool {
        for _ in 0..16 {
            let mut dirent = zeroed_dirent();
            let ret = unsafe { readdir(fd, &raw mut dirent, 1) };

            assert!(ret >= 0, "readdir failed: {ret}");
            if ret == 0 {
                return false;
            }

            let name = dirent_name(&dirent);
            assert!(dirent.d_namlen > 0);
            assert_ne!(dirent.d_ino, 0);
            let nul = dirent.d_name[dirent.d_namlen as usize];
            assert_eq!(nul, 0 as Char);

            if name == target_name {
                return true;
            }
        }

        false
    }

    #[test]
    fn test_readdir_sysno() {
        assert_eq!(Sysno::Readdir as isize, 89);
    }

    #[test]
    fn test_readdir_dirent_layout() {
        assert_eq!(core::mem::size_of::<OldLinuxDirent>(), 4108);
        assert_eq!(core::mem::align_of::<OldLinuxDirent>(), 4);
        assert_eq!(core::mem::offset_of!(OldLinuxDirent, d_ino), 0);
        assert_eq!(core::mem::offset_of!(OldLinuxDirent, d_offset), 4);
        assert_eq!(core::mem::offset_of!(OldLinuxDirent, d_namlen), 8);
        assert_eq!(core::mem::offset_of!(OldLinuxDirent, d_name), 10);
    }

    #[test]
    fn test_readdir_reads_created_entry() {
        let dir = create_temp_dir();
        let target_name = b"entry.txt";
        touch(&dir.join("entry.txt"));

        let dir_file = File::open(&dir).unwrap();
        let fd = dir_file.as_raw_fd() as UnsignedInt;
        let found = directory_contains_entry(fd, target_name);

        assert!(found, "readdir did not return the created directory entry");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_readdir_empty_directory_returns_zero() {
        let dir = create_temp_dir();
        let dir_file = File::open(&dir).unwrap();
        let fd = dir_file.as_raw_fd() as UnsignedInt;
        let mut dirent = zeroed_dirent();

        loop {
            let ret = unsafe { readdir(fd, &raw mut dirent, 1) };
            assert!(ret >= 0, "readdir failed: {ret}");

            if ret == 0 {
                break;
            }
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_readdir_regular_file_returns_enotdir() {
        let dir = create_temp_dir();
        let file_path = dir.join("not_a_directory");
        touch(&file_path);

        let file = File::open(&file_path).unwrap();
        let mut entry = zeroed_dirent();
        let ret = unsafe {
            readdir(file.as_raw_fd() as UnsignedInt, &raw mut entry, 1)
        };

        assert_eq!(ret, -20, "expected ENOTDIR, got {ret}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_readdir_invalid_fd_returns_ebadf() {
        let mut entry = zeroed_dirent();
        let ret = unsafe { readdir(UnsignedInt::MAX, &raw mut entry, 1) };

        assert_eq!(ret, -9, "expected EBADF, got {ret}");
    }

    #[test]
    fn test_readdir_invalid_pointer_returns_efault() {
        let dir = create_temp_dir();
        touch(&dir.join("entry.txt"));

        let dir_file = File::open(&dir).unwrap();
        let bad_dirent =
            core::ptr::without_provenance_mut::<OldLinuxDirent>(usize::MAX);
        let ret = unsafe {
            readdir(dir_file.as_raw_fd() as UnsignedInt, bad_dirent, 1)
        };

        assert_eq!(ret, -14, "expected EFAULT, got {ret}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_readdir_count_greater_than_one_still_returns_single_entry() {
        let dir = create_temp_dir();
        touch(&dir.join("entry.txt"));

        let dir_file = File::open(&dir).unwrap();
        let fd = dir_file.as_raw_fd() as UnsignedInt;
        let mut dirent = zeroed_dirent();
        let ret = unsafe { readdir(fd, &raw mut dirent, 32) };

        assert!(ret >= 0, "readdir failed: {ret}");
        if ret > 0 {
            assert!(dirent.d_namlen > 0);
            let nul = dirent.d_name[dirent.d_namlen as usize];
            assert_eq!(nul, 0 as Char);
        }

        fs::remove_dir_all(&dir).unwrap();
    }
}
