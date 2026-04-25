use celer_system_linux_ctypes::{SizeT, Void};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`munmap`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MunmapError {
    /// `EINVAL`.
    Einval,
    /// Another errno returned by unmapping implementation details.
    Other(Errno),
}

impl MunmapError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            errno => Self::Other(errno),
        }
    }
}

/// Remove mappings in the supplied address range.
///
/// This wrapper takes the start address as a pointer, keeps the raw length
/// argument, and maps the raw return into `Result<(), MunmapError>`.
///
/// On success, the kernel has removed mappings covering the requested range.
///
/// See [`sys::munmap`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// The caller must ensure unmapping this range does not invalidate live Rust
/// references, pointers whose pointees must remain valid, or allocator
/// assumptions in the current process.
///
/// # Errors
/// - [`MunmapError::Einval`]: the address or length is invalid.
/// - [`MunmapError::Other`]: another errno from unmapping implementation
///   details.
pub unsafe fn munmap(addr: *mut Void, len: SizeT) -> Result<(), MunmapError> {
    // SAFETY: the caller upholds the process-memory invariants required by
    // this unmapping operation.
    let ret = unsafe { sys::munmap(addr.addr() as _, len) };

    unit_from_ret(ret as isize, MunmapError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{SizeT, UnsignedInt, UnsignedLong, Void};

    use crate::Errno;

    use super::{MunmapError, munmap};
    use crate::mmap;

    const PROT_READ: UnsignedLong = 0x1;
    const PROT_WRITE: UnsignedLong = 0x2;
    const MAP_PRIVATE: UnsignedLong = 0x02;
    const MAP_ANONYMOUS: UnsignedLong = 0x20;

    #[test]
    fn test_munmap_ok() {
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
    fn test_munmap_unaligned_addr_returns_einval() {
        let err =
            unsafe { munmap(ptr::without_provenance_mut::<Void>(1), 4096) }
                .unwrap_err();

        assert_eq!(err, MunmapError::Einval);
    }

    #[test]
    fn test_munmap_error_mapping() {
        assert_eq!(MunmapError::from_errno(Errno::Einval), MunmapError::Einval);
        assert_eq!(
            MunmapError::from_errno(Errno::Enomem),
            MunmapError::Other(Errno::Enomem)
        );
    }
}
