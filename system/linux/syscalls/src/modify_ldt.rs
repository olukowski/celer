use celer_system_linux_ctypes::{Int, UnsignedLong, Void};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`modify_ldt`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModifyLdtError {
    /// `EINVAL`.
    Einval,
    /// `EFAULT`.
    Efault,
    /// `ENOMEM`.
    Enomem,
    /// `EINTR`.
    Eintr,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by current x86 LDT support.
    Other(Errno),
}

impl ModifyLdtError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            Errno::Enomem => Self::Enomem,
            Errno::Eintr => Self::Eintr,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Read or update the calling task's x86 local descriptor table state.
///
/// This wrapper preserves the raw `func`, pointer, and byte-count arguments
/// because the pointer direction depends on `func`, and maps the raw return
/// into `Result<Int, ModifyLdtError>`.
///
/// On success, returns the kernel byte count for read functions or `0` for
/// write functions.
///
/// See [`sys::modify_ldt`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// The caller must ensure `ptr` is valid for the selected `func`: writable for
/// read functions, readable for write functions, and non-aliasing for any
/// kernel writes performed during the syscall.
///
/// # Errors
/// - [`ModifyLdtError::Einval`]: invalid function-specific LDT data.
/// - [`ModifyLdtError::Efault`]: the user buffer is not accessible.
/// - [`ModifyLdtError::Enomem`]: the kernel could not allocate LDT state.
/// - [`ModifyLdtError::Eintr`]: the write path was interrupted.
/// - [`ModifyLdtError::Enosys`]: the function or syscall is not supported.
/// - [`ModifyLdtError::Other`]: another errno from current x86 LDT support.
#[cfg(target_arch = "x86")]
pub unsafe fn modify_ldt(
    func: Int,
    ptr: *mut Void,
    bytecount: UnsignedLong,
) -> Result<Int, ModifyLdtError> {
    // SAFETY: the caller upholds the function-specific pointer contract.
    let ret = unsafe { sys::modify_ldt(func, ptr, bytecount) };

    result_from_ret(ret as isize, |ret| ret as Int, ModifyLdtError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, UnsignedLong};

    use crate::Errno;

    use super::{ModifyLdtError, modify_ldt};

    #[test]
    fn test_modify_ldt_zero_length_read() {
        let got = unsafe { modify_ldt(2 as Int, core::ptr::null_mut(), 0) };

        assert!(matches!(got, Ok(0) | Err(ModifyLdtError::Enosys)));
    }

    #[test]
    fn test_modify_ldt_unsupported_func() {
        let got = unsafe {
            modify_ldt(Int::MAX, core::ptr::null_mut(), 0 as UnsignedLong)
        };

        assert_eq!(got, Err(ModifyLdtError::Enosys));
    }

    #[test]
    fn test_modify_ldt_error_mapping() {
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Einval),
            ModifyLdtError::Einval
        );
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Efault),
            ModifyLdtError::Efault
        );
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Enomem),
            ModifyLdtError::Enomem
        );
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Eintr),
            ModifyLdtError::Eintr
        );
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Enosys),
            ModifyLdtError::Enosys
        );
        assert_eq!(
            ModifyLdtError::from_errno(Errno::Eio),
            ModifyLdtError::Other(Errno::Eio)
        );
    }
}
