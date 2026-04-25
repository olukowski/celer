use celer_system_linux_ctypes::Vm86Struct;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`vm86`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Vm86Error {
    Eperm,
    Efault,
    Einval,
    Enomem,
    Enosys,
    Other(Errno),
}

impl Vm86Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Enter x86 virtual-8086 mode.
///
/// This wrapper replaces the raw pointer with `&mut Vm86Struct` and maps
/// errno-shaped failures into `Result<(), Vm86Error>`.
///
/// See [`sys::vm86`] for kernel behavior, ABI notes, reachable errors, and
/// source references.
///
/// # Safety
/// `v86` must remain valid for the entire vm86 session, and entering vm86 mode
/// can transfer control according to the register image stored in `v86`.
///
/// # Errors
/// - [`Vm86Error::Eperm`]: the kernel rejects the caller or nested vm86 state.
/// - [`Vm86Error::Efault`]: the kernel could not read the state record.
/// - [`Vm86Error::Einval`]: the state record contains rejected flags.
/// - [`Vm86Error::Enomem`]: the kernel could not allocate vm86 bookkeeping.
/// - [`Vm86Error::Enosys`]: the running kernel was built without vm86 support.
/// - [`Vm86Error::Other`]: any other errno reported by the raw ABI.
#[cfg(target_arch = "x86")]
pub unsafe fn vm86(v86: &mut Vm86Struct) -> Result<(), Vm86Error> {
    let ret = unsafe { sys::vm86(v86) };
    unit_from_ret(ret as isize, Vm86Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::Vm86Error;

    #[test]
    fn test_vm86_error_mapping() {
        assert_eq!(Vm86Error::from_errno(Errno::Eperm), Vm86Error::Eperm);
        assert_eq!(Vm86Error::from_errno(Errno::Efault), Vm86Error::Efault);
        assert_eq!(Vm86Error::from_errno(Errno::Einval), Vm86Error::Einval);
        assert_eq!(Vm86Error::from_errno(Errno::Enomem), Vm86Error::Enomem);
        assert_eq!(Vm86Error::from_errno(Errno::Enosys), Vm86Error::Enosys);
        assert_eq!(
            Vm86Error::from_errno(Errno::Eio),
            Vm86Error::Other(Errno::Eio)
        );
    }
}
