use core::ffi::CStr;

use celer_system_linux_ctypes::UnsignedInt;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`delete_module`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeleteModuleError {
    /// `EPERM`.
    Eperm,
    /// Linux 1.0 `E2BIG` when `module_name` exceeds `MOD_MAX_NAME`.
    E2big,
    /// `ENOENT`.
    Enoent,
    /// Current kernels' `EFAULT` for an unreadable module name pointer.
    Efault,
    /// `EINTR`.
    Eintr,
    /// Current kernels' `EWOULDBLOCK`.
    Ewouldblock,
    /// Current kernels' `EBUSY`.
    Ebusy,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by the raw syscall.
    Other(Errno),
}

impl DeleteModuleError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::E2big => Self::E2big,
            Errno::Enoent => Self::Enoent,
            Errno::Efault => Self::Efault,
            Errno::Eintr => Self::Eintr,
            Errno::Eagain | Errno::Ewouldblock => Self::Ewouldblock,
            Errno::Ebusy => Self::Ebusy,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Unload a kernel module by name.
///
/// This safe wrapper converts the raw module-name pointer into
/// `Option<&CStr>` and maps the raw `delete_module(2)` return value into
/// `Result<(), DeleteModuleError>`. It preserves the kernel-facing `flags`
/// argument as-is.
///
/// Passing `Some(name)` unloads the named module. Passing `None` preserves the
/// historical Linux 1.0 null-name case and passes a null pointer to the raw
/// syscall.
///
/// On success, returns `Ok(())` after the kernel has completed the unload
/// path.
///
/// See [`sys::delete_module`] for kernel behavior, reachable errors, and
/// source references.
///
/// # Errors
/// - [`DeleteModuleError::Eperm`]: the caller lacks permission to unload
///   modules, or current kernels have module loading disabled globally.
/// - [`DeleteModuleError::E2big`]: Linux 1.0 scanned `module_name` without
///   finding a trailing NUL before `MOD_MAX_NAME`.
/// - [`DeleteModuleError::Enoent`]: no loaded module matches `module_name`, or
///   current kernels reject an empty or too-long name.
/// - [`DeleteModuleError::Efault`]: current kernels cannot read the module
///   name from user memory.
/// - [`DeleteModuleError::Eintr`]: a current kernel was interrupted while
///   waiting for `module_mutex`.
/// - [`DeleteModuleError::Ewouldblock`]: a current kernel found dependent
///   modules or refused to stop the target without force.
/// - [`DeleteModuleError::Ebusy`]: a current kernel found the module already
///   unloading, not live, or lacking an unload path that force flags can use.
/// - [`DeleteModuleError::Enosys`]: the syscall slot is unimplemented on the
///   running kernel.
/// - [`DeleteModuleError::Other`]: another errno.
pub fn delete_module(
    module_name: Option<&CStr>,
    flags: UnsignedInt,
) -> Result<(), DeleteModuleError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pointer for the
    // duration of the syscall, and `None` maps to a null pointer.
    let ret = unsafe {
        sys::delete_module(
            module_name.map_or(core::ptr::null(), CStr::as_ptr),
            flags,
        )
    };

    unit_from_ret(ret as isize, DeleteModuleError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use celer_system_linux_ctypes::UnsignedInt;

    use crate::Errno;

    use super::{DeleteModuleError, delete_module};

    #[test]
    fn test_delete_module_missing_module() {
        let name =
            CString::new("definitely_not_a_loaded_celer_module").unwrap();

        let err = delete_module(Some(name.as_c_str()), 0 as UnsignedInt)
            .expect_err("missing module should not unload successfully");

        assert!(matches!(
            err,
            DeleteModuleError::Eperm
                | DeleteModuleError::Enoent
                | DeleteModuleError::Enosys
        ));
    }

    #[test]
    fn test_delete_module_error_mapping() {
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Eperm),
            DeleteModuleError::Eperm
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::E2big),
            DeleteModuleError::E2big
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Enoent),
            DeleteModuleError::Enoent
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Efault),
            DeleteModuleError::Efault
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Eintr),
            DeleteModuleError::Eintr
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Eagain),
            DeleteModuleError::Ewouldblock
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Ewouldblock),
            DeleteModuleError::Ewouldblock
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Ebusy),
            DeleteModuleError::Ebusy
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Enosys),
            DeleteModuleError::Enosys
        );
        assert_eq!(
            DeleteModuleError::from_errno(Errno::Enomem),
            DeleteModuleError::Other(Errno::Enomem)
        );
    }

    #[test]
    fn test_delete_module_none() {
        let err = delete_module(None, 0 as UnsignedInt).expect_err(
            "delete_module(None, 0) should fail on current kernels unless Linux 1.0 semantics are active",
        );

        assert!(matches!(
            err,
            DeleteModuleError::Eperm
                | DeleteModuleError::Efault
                | DeleteModuleError::Enosys
        ));
    }
}
