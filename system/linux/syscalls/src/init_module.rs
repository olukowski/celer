use core::ffi::CStr;

#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::{ModRoutines, UnsignedInt};
use celer_system_linux_ctypes::{UnsignedLong, Void};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`init_module`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InitModuleError {
    /// `EPERM`.
    Eperm,
    /// `ENOENT`.
    Enoent,
    /// `EINVAL`.
    Einval,
    /// `EBUSY`.
    Ebusy,
    /// `EINTR`.
    Eintr,
    /// `EFAULT`.
    Efault,
    /// `ENOMEM`.
    Enomem,
    /// `ENOEXEC`.
    Enoexec,
    /// `EEXIST`.
    Eexist,
    /// `EBADMSG`.
    Ebadmsg,
    /// `EKEYREJECTED`.
    Ekeyrejected,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by delegated policy or object-specific loader
    /// paths.
    Other(Errno),
}

impl InitModuleError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Enoent => Self::Enoent,
            Errno::Einval => Self::Einval,
            Errno::Ebusy => Self::Ebusy,
            Errno::Eintr => Self::Eintr,
            Errno::Efault => Self::Efault,
            Errno::Enomem => Self::Enomem,
            Errno::Enoexec => Self::Enoexec,
            Errno::Eexist => Self::Eexist,
            Errno::Ebadmsg => Self::Ebadmsg,
            Errno::Ekeyrejected => Self::Ekeyrejected,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Load a kernel module image from user memory.
///
/// This safe wrapper converts the raw module-image pointer and length into a
/// shared byte slice and converts the module-parameter pointer into `&CStr`.
/// It maps the raw `init_module(2)` return value into
/// `Result<(), InitModuleError>`.
///
/// On success, returns `Ok(())` after the kernel has accepted the copied
/// module image and completed the module's load path.
///
/// See [`sys::init_module`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`InitModuleError::Eperm`]: the caller lacks permission to load modules,
///   module loading is disabled, or policy rejects the load.
/// - [`InitModuleError::Enoent`]: the kernel could not resolve a required
///   non-weak symbol.
/// - [`InitModuleError::Einval`]: the kernel rejected malformed module state
///   or arguments after the syscall reached those checks.
/// - [`InitModuleError::Ebusy`]: the kernel detected an in-progress duplicate
///   load or a still-loading symbol owner.
/// - [`InitModuleError::Eintr`]: the duplicate-load wait was interrupted.
/// - [`InitModuleError::Efault`]: the kernel could not read the module image
///   or parameter string from user memory.
/// - [`InitModuleError::Enomem`]: the kernel could not allocate loader
///   memory.
/// - [`InitModuleError::Enoexec`]: the kernel rejected the supplied bytes as
///   an invalid module image.
/// - [`InitModuleError::Eexist`]: the same module is already live.
/// - [`InitModuleError::Ebadmsg`]: the module signature trailer was malformed.
/// - [`InitModuleError::Ekeyrejected`]: signature enforcement rejected the
///   module image.
/// - [`InitModuleError::Enosys`]: the running kernel left this syscall slot
///   unimplemented.
/// - [`InitModuleError::Other`]: another loader, filesystem, or security-hook
///   errno.
pub fn init_module(image: &[u8], uargs: &CStr) -> Result<(), InitModuleError> {
    // SAFETY: `image` and `CStr` provide valid user-memory pointers for the
    // duration of the syscall.
    let ret = unsafe {
        sys::init_module(
            image.as_ptr().cast::<Void>(),
            image.len() as UnsignedLong,
            uargs.as_ptr(),
        )
    };

    unit_from_ret(ret as isize, InitModuleError::from_errno)
}

/// Initialize a Linux 1.0 module allocation.
///
/// This safe wrapper mirrors the historical
/// [`sys::linux_1_0::init_module`] ABI by taking NUL-terminated module names,
/// a shared module-code slice, and a shared [`ModRoutines`] record.
///
/// See [`sys::linux_1_0::init_module`] for kernel behavior, reachable errors,
/// and source references.
///
/// # Errors
/// - [`InitModuleError::Eperm`]: the caller lacks permission.
/// - [`InitModuleError::Enoent`]: no matching module allocation exists.
/// - [`InitModuleError::Einval`]: the code image is too large for the target
///   allocation.
/// - [`InitModuleError::Ebusy`]: the module init routine rejected the load.
/// - [`InitModuleError::Other`]: another historical errno.
#[cfg(target_arch = "x86")]
pub fn init_module_1_0(
    module_name: &CStr,
    code: &[u8],
    routines: &ModRoutines,
) -> Result<(), InitModuleError> {
    let codesize = UnsignedInt::try_from(code.len())
        .map_err(|_| InitModuleError::Einval)?;
    // SAFETY: the wrappers provide readable pointers for the historical ABI.
    let ret = unsafe {
        sys::linux_1_0::init_module(
            module_name.as_ptr(),
            code.as_ptr().cast::<Void>(),
            codesize,
            routines,
        )
    };
    unit_from_ret(ret as isize, InitModuleError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::ModRoutines;

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::init_module_1_0;
    use super::{InitModuleError, init_module};

    #[test]
    fn test_init_module_empty_image() {
        let err = init_module(&[], c"")
            .expect_err("an empty module image should not load successfully");

        assert!(matches!(
            err,
            InitModuleError::Eperm
                | InitModuleError::Enoexec
                | InitModuleError::Enosys
        ));
    }

    #[test]
    fn test_init_module_error_mapping() {
        assert_eq!(
            InitModuleError::from_errno(Errno::Eperm),
            InitModuleError::Eperm
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Enoent),
            InitModuleError::Enoent
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Einval),
            InitModuleError::Einval
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Ebusy),
            InitModuleError::Ebusy
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Eintr),
            InitModuleError::Eintr
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Efault),
            InitModuleError::Efault
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Enomem),
            InitModuleError::Enomem
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Enoexec),
            InitModuleError::Enoexec
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Eexist),
            InitModuleError::Eexist
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Ebadmsg),
            InitModuleError::Ebadmsg
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Ekeyrejected),
            InitModuleError::Ekeyrejected
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Enosys),
            InitModuleError::Enosys
        );
        assert_eq!(
            InitModuleError::from_errno(Errno::Eio),
            InitModuleError::Other(Errno::Eio)
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_init_module_1_0_missing_allocation() {
        let routines = ModRoutines {
            init: 0,
            cleanup: 0,
        };
        let err = init_module_1_0(c"celer_missing_module", &[], &routines)
            .expect_err("missing Linux 1.0 module allocation should fail");

        assert!(matches!(
            err,
            InitModuleError::Eperm | InitModuleError::Enoent
        ));
    }
}
