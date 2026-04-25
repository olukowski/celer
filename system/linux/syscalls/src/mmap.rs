use celer_system_linux_ctypes::{SizeT, UnsignedInt, UnsignedLong, Void};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`mmap`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmapError {
    /// `EACCES`.
    Eacces,
    /// `EBADF`.
    Ebadf,
    /// `EINVAL`.
    Einval,
    /// `ENODEV`.
    Enodev,
    /// `ENOMEM`.
    Enomem,
    /// `ENOEXEC`.
    Enoexec,
    /// Another errno returned by delegated file, filesystem, driver, or
    /// security-hook mapping code.
    Other(Errno),
}

impl MmapError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eacces => Self::Eacces,
            Errno::Ebadf => Self::Ebadf,
            Errno::Einval => Self::Einval,
            Errno::Enodev => Self::Enodev,
            Errno::Enomem => Self::Enomem,
            Errno::Enoexec => Self::Enoexec,
            errno => Self::Other(errno),
        }
    }
}

/// Create a memory mapping.
///
/// This wrapper keeps the raw `mmap(addr, len, prot, flags, fd, offset)` shape
/// and maps the address-valued raw return into `Result<*mut Void, MmapError>`.
///
/// On success, returns the mapped address chosen by the kernel.
///
/// See [`sys::mmap`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// The caller must ensure the requested mapping operation does not invalidate
/// live Rust references, pointers whose pointees must remain valid, or
/// allocator assumptions in the current process.
///
/// # Errors
/// - [`MmapError::Eacces`]: the requested mapping conflicts with file access.
/// - [`MmapError::Ebadf`]: `fd` is not an open file descriptor for a
///   non-anonymous mapping.
/// - [`MmapError::Einval`]: the range, flags, protection, address, or offset
///   is invalid.
/// - [`MmapError::Enodev`]: the target file does not support mappings.
/// - [`MmapError::Enomem`]: the kernel could not allocate or find the mapping.
/// - [`MmapError::Enoexec`]: a historical filesystem mapper rejected the inode.
/// - [`MmapError::Other`]: delegated file, filesystem, driver, or
///   security-hook mapping error.
pub unsafe fn mmap(
    addr: *mut Void,
    len: SizeT,
    prot: UnsignedLong,
    flags: UnsignedLong,
    fd: UnsignedInt,
    offset: UnsignedLong,
) -> Result<*mut Void, MmapError> {
    // SAFETY: the caller upholds the process-memory invariants required by
    // this mapping operation.
    let ret = unsafe { sys::mmap(addr, len, prot, flags, fd, offset) };

    result_from_ret(ret as isize, |ret| ret as *mut Void, MmapError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{SizeT, UnsignedInt, UnsignedLong};

    use crate::Errno;

    use super::{MmapError, mmap};
    use crate::munmap;

    const PROT_READ: UnsignedLong = 0x1;
    const PROT_WRITE: UnsignedLong = 0x2;
    const MAP_PRIVATE: UnsignedLong = 0x02;
    const MAP_ANONYMOUS: UnsignedLong = 0x20;

    #[test]
    fn test_mmap_anonymous_ok() {
        let len = 4096 as SizeT;

        let mapping = unsafe {
            mmap(
                ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0 as UnsignedInt,
                0,
            )
        }
        .expect("anonymous mmap should succeed");

        unsafe { munmap(mapping, len).expect("munmap should succeed") };
    }

    #[test]
    fn test_mmap_bad_fd() {
        let err = unsafe {
            mmap(
                ptr::null_mut(),
                4096 as SizeT,
                PROT_READ,
                MAP_PRIVATE,
                UnsignedInt::MAX,
                0,
            )
        }
        .unwrap_err();

        assert_eq!(err, MmapError::Ebadf);
    }

    #[test]
    fn test_mmap_error_mapping() {
        assert_eq!(MmapError::from_errno(Errno::Eacces), MmapError::Eacces);
        assert_eq!(MmapError::from_errno(Errno::Ebadf), MmapError::Ebadf);
        assert_eq!(MmapError::from_errno(Errno::Einval), MmapError::Einval);
        assert_eq!(MmapError::from_errno(Errno::Enodev), MmapError::Enodev);
        assert_eq!(MmapError::from_errno(Errno::Enomem), MmapError::Enomem);
        assert_eq!(MmapError::from_errno(Errno::Enoexec), MmapError::Enoexec);
        assert_eq!(
            MmapError::from_errno(Errno::Eio),
            MmapError::Other(Errno::Eio)
        );
    }
}
