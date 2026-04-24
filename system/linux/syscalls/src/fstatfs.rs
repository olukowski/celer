use core::mem::MaybeUninit;

#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Statfs as Linux10Statfs;
use celer_system_linux_ctypes::{Statfs, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`fstatfs`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FstatfsError {
    /// `EFAULT`.
    Efault,
    /// `EBADF`.
    Ebadf,
    /// `ENOENT` from the Linux 1.0 x86 ABI when `fd` has no inode.
    Enoent,
    /// `ENOSYS`.
    Enosys,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated filesystem-specific `statfs` work.
    Other(Errno),
}

impl FstatfsError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Ebadf => Self::Ebadf,
            Errno::Enoent => Self::Enoent,
            Errno::Enosys => Self::Enosys,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Return filesystem status information for an open file descriptor.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Statfs>` and maps the raw syscall return into
/// `Result<(), FstatfsError>`.
///
/// On success, the kernel has initialized `buf` with the filesystem status for
/// the open file referenced by `fd`.
///
/// See [`sys::fstatfs`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`FstatfsError::Efault`]: the kernel could not write the output
///   structure.
/// - [`FstatfsError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FstatfsError::Enosys`]: the backing filesystem does not implement a
///   `statfs` hook.
/// - [`FstatfsError::Eoverflow`]: the current ABI could not represent the
///   filesystem statistics.
/// - [`FstatfsError::Other`]: delegated filesystem-specific `statfs` error.
pub fn fstatfs(
    fd: UnsignedInt,
    buf: &mut MaybeUninit<Statfs>,
) -> Result<(), FstatfsError> {
    // SAFETY: `MaybeUninit<Statfs>` provides a valid, writable output buffer
    // for one `Statfs`.
    let ret = unsafe { sys::fstatfs(fd, buf.as_mut_ptr()) };

    result_from_ret(ret as isize, |_| (), FstatfsError::from_errno)
}

/// Return filesystem status through the Linux 1.0 `sys_fstatfs` ABI.
///
/// This safe wrapper mirrors [`fstatfs`], but uses
/// `&mut MaybeUninit<Linux10Statfs>` for the Linux 1.0 layout exposed from
/// [`crate::linux_1_0`].
///
/// See [`sys::linux_1_0::fstatfs`] for historical kernel behavior and source
/// references.
///
/// # Errors
/// - [`FstatfsError::Efault`]: the kernel could not write the output
///   structure.
/// - [`FstatfsError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`FstatfsError::Enoent`]: Linux 1.0 found no inode for the open file.
/// - [`FstatfsError::Enosys`]: the backing filesystem does not implement a
///   `statfs` hook.
#[cfg(target_arch = "x86")]
pub fn fstatfs_1_0(
    fd: UnsignedInt,
    buf: &mut MaybeUninit<Linux10Statfs>,
) -> Result<(), FstatfsError> {
    // SAFETY: `MaybeUninit<Linux10Statfs>` provides a valid, writable output
    // buffer for one Linux 1.0 `struct statfs`.
    let ret = unsafe { sys::linux_1_0::fstatfs(fd, buf.as_mut_ptr()) };

    result_from_ret(ret as isize, |_| (), FstatfsError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs::File, mem::MaybeUninit, os::fd::AsRawFd as _};

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::fstatfs_1_0;
    use super::{FstatfsError, fstatfs};

    #[test]
    fn test_fstatfs_ok() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as u32;
        let mut statfs = MaybeUninit::uninit();

        fstatfs(fd, &mut statfs).expect("fstatfs should succeed for /");
        let statfs = unsafe { statfs.assume_init() };

        assert!(
            statfs.f_bsize > 0,
            "expected positive block size: {statfs:?}"
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_fstatfs_ok() {
        let file = File::open("/").unwrap();
        let fd = file.as_raw_fd() as u32;
        let mut statfs = MaybeUninit::uninit();

        fstatfs_1_0(fd, &mut statfs)
            .expect("linux_1_0::fstatfs should succeed for /");
        let statfs = unsafe { statfs.assume_init() };

        assert!(
            statfs.f_bsize > 0,
            "expected positive block size from Linux 1.0 layout: {statfs:?}"
        );
    }

    #[test]
    fn test_fstatfs_invalid_fd() {
        let mut statfs = MaybeUninit::uninit();

        assert_eq!(fstatfs(u32::MAX, &mut statfs), Err(FstatfsError::Ebadf));
    }

    #[test]
    fn test_fstatfs_error_mapping() {
        assert_eq!(
            FstatfsError::from_errno(Errno::Efault),
            FstatfsError::Efault
        );
        assert_eq!(FstatfsError::from_errno(Errno::Ebadf), FstatfsError::Ebadf);
        assert_eq!(
            FstatfsError::from_errno(Errno::Enoent),
            FstatfsError::Enoent
        );
        assert_eq!(
            FstatfsError::from_errno(Errno::Enosys),
            FstatfsError::Enosys
        );
        assert_eq!(
            FstatfsError::from_errno(Errno::Eoverflow),
            FstatfsError::Eoverflow
        );
        assert_eq!(
            FstatfsError::from_errno(Errno::Enomem),
            FstatfsError::Other(Errno::Enomem)
        );
    }
}
