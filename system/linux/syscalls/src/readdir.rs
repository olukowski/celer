use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{OldLinuxDirent, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`readdir`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReaddirError {
    /// `EBADF`.
    Ebadf,
    /// `ENOTDIR`.
    Enotdir,
    /// `EFAULT`.
    Efault,
    /// `ENOENT`.
    Enoent,
    /// `EIO`.
    Eio,
    /// `EOVERFLOW`.
    Eoverflow,
    /// Another errno returned by delegated filesystem code.
    Other(Errno),
}

impl ReaddirError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Enotdir => Self::Enotdir,
            Errno::Efault => Self::Efault,
            Errno::Enoent => Self::Enoent,
            Errno::Eio => Self::Eio,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Read one directory entry through the legacy x86 `readdir` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldLinuxDirent>` and maps the raw syscall return value
/// into `Result<usize, ReaddirError>`.
///
/// On success, returns the raw nonnegative result from the kernel. Current
/// kernels return `1` when one entry was emitted and `0` at end of directory.
///
/// See [`sys::readdir`] for kernel behavior, reachable errors, ABI layout, and
/// source references.
///
/// # Errors
/// - [`ReaddirError::Ebadf`]: `fd` is not an open usable descriptor.
/// - [`ReaddirError::Enotdir`]: `fd` does not refer to a directory.
/// - [`ReaddirError::Efault`]: the kernel could not write the output buffer.
/// - [`ReaddirError::Enoent`]: delegated directory reading reported a missing
///   entry or invalid directory position.
/// - [`ReaddirError::Eio`]: delegated directory reading failed with I/O error.
/// - [`ReaddirError::Eoverflow`]: returned metadata did not fit the ABI.
/// - [`ReaddirError::Other`]: delegated filesystem error.
#[cfg(target_arch = "x86")]
pub fn readdir(
    fd: UnsignedInt,
    dirent: &mut MaybeUninit<OldLinuxDirent>,
    count: UnsignedInt,
) -> Result<usize, ReaddirError> {
    // SAFETY: `MaybeUninit<OldLinuxDirent>` provides writable storage for one
    // kernel-initialized legacy directory entry.
    let ret = unsafe { sys::readdir(fd, dirent.as_mut_ptr(), count) };

    result_from_ret(ret as isize, |ret| ret as usize, ReaddirError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::mem::MaybeUninit;

    use crate::Errno;

    use super::{ReaddirError, readdir};

    #[test]
    fn test_readdir_invalid_fd() {
        let mut dirent = MaybeUninit::uninit();

        assert_eq!(readdir(u32::MAX, &mut dirent, 1), Err(ReaddirError::Ebadf));
    }

    #[test]
    fn test_readdir_error_mapping() {
        assert_eq!(ReaddirError::from_errno(Errno::Ebadf), ReaddirError::Ebadf);
        assert_eq!(
            ReaddirError::from_errno(Errno::Enotdir),
            ReaddirError::Enotdir
        );
        assert_eq!(
            ReaddirError::from_errno(Errno::Efault),
            ReaddirError::Efault
        );
        assert_eq!(
            ReaddirError::from_errno(Errno::Enoent),
            ReaddirError::Enoent
        );
        assert_eq!(ReaddirError::from_errno(Errno::Eio), ReaddirError::Eio);
        assert_eq!(
            ReaddirError::from_errno(Errno::Eoverflow),
            ReaddirError::Eoverflow
        );
        assert_eq!(
            ReaddirError::from_errno(Errno::Eperm),
            ReaddirError::Other(Errno::Eperm)
        );
    }
}
