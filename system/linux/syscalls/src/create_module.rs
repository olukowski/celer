use core::ffi::CStr;
use core::ptr::{self, NonNull};

use celer_system_linux_ctypes::{UnsignedLong, Void};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`create_module`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreateModuleError {
    /// `EPERM`.
    Eperm,
    /// `EINVAL`.
    Einval,
    /// `E2BIG`.
    E2big,
    /// `EEXIST`.
    Eexist,
    /// `ENOMEM`.
    Enomem,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by the raw syscall.
    Other(Errno),
}

impl CreateModuleError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Einval => Self::Einval,
            Errno::E2big => Self::E2big,
            Errno::Eexist => Self::Eexist,
            Errno::Enomem => Self::Enomem,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Allocate a Linux 1.0 kernel-module slot by name and requested image size.
///
/// This safe wrapper takes a NUL-terminated module name and maps the raw
/// address-or-errno return into `Result<NonNull<Void>, CreateModuleError>`.
///
/// On success, returns the non-null module-image base address reported by the
/// kernel as an opaque pointer value.
///
/// See [`sys::create_module`] for kernel behavior, reachable errors, and
/// source references.
///
/// # Errors
/// - [`CreateModuleError::Eperm`]: the caller lacks Linux 1.0 superuser
///   privilege.
/// - [`CreateModuleError::Einval`]: `size` is zero.
/// - [`CreateModuleError::E2big`]: `module_name` exceeds Linux 1.0's fixed
///   module-name buffer.
/// - [`CreateModuleError::Eexist`]: a live module with the same name already
///   exists.
/// - [`CreateModuleError::Enomem`]: kernel allocation for the module record or
///   image failed.
/// - [`CreateModuleError::Enosys`]: current x86 kernels keep this historical
///   syscall slot unimplemented.
/// - [`CreateModuleError::Other`]: another errno.
pub fn create_module(
    module_name: &CStr,
    size: UnsignedLong,
) -> Result<NonNull<Void>, CreateModuleError> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pointer for the
    // duration of the syscall.
    let ret = unsafe { sys::create_module(module_name.as_ptr(), size) };

    create_module_from_ret(ret)
}

fn create_module_from_ret(
    ret: UnsignedLong,
) -> Result<NonNull<Void>, CreateModuleError> {
    result_from_ret(
        ret as isize,
        create_module_success,
        CreateModuleError::from_errno,
    )
}

fn create_module_success(ret: isize) -> NonNull<Void> {
    // SAFETY: a successful Linux 1.0 `create_module` return is the base
    // address of the allocated module image.
    unsafe {
        NonNull::new_unchecked(ptr::without_provenance_mut::<Void>(
            ret as usize,
        ))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use celer_system_linux_ctypes::UnsignedLong;

    use crate::Errno;

    use super::{CreateModuleError, create_module, create_module_from_ret};

    #[test]
    fn test_create_module_zero_size() {
        let name = CString::new("celer_test_module").unwrap();

        match create_module(name.as_c_str(), 0 as UnsignedLong) {
            Err(CreateModuleError::Einval | CreateModuleError::Enosys) => {}
            other => panic!("unexpected create_module result: {other:?}"),
        }
    }

    #[test]
    fn test_create_module_current_kernel_enosys() {
        let name = CString::new("celer_test_module").unwrap();

        let err = create_module(name.as_c_str(), 4096 as UnsignedLong)
            .expect_err(
                "current x86 kernels should route create_module to ENOSYS",
            );
        assert_eq!(err, CreateModuleError::Enosys);
    }

    #[test]
    fn test_create_module_success_mapping() {
        let ptr = create_module_from_ret(4096 as UnsignedLong).unwrap();

        assert_eq!(ptr.addr().get(), 4096);
    }

    #[test]
    fn test_create_module_error_mapping() {
        assert_eq!(
            CreateModuleError::from_errno(Errno::Eperm),
            CreateModuleError::Eperm
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::Einval),
            CreateModuleError::Einval
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::E2big),
            CreateModuleError::E2big
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::Eexist),
            CreateModuleError::Eexist
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::Enomem),
            CreateModuleError::Enomem
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::Enosys),
            CreateModuleError::Enosys
        );
        assert_eq!(
            CreateModuleError::from_errno(Errno::Enoent),
            CreateModuleError::Other(Errno::Enoent)
        );
    }
}
