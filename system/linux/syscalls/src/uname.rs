use core::mem::MaybeUninit;

use celer_system_linux_ctypes::NewUtsname;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::{OldOldUtsname, OldUtsname};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`oldolduname`].
#[cfg(target_arch = "x86")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldoldunameError {
    /// `EFAULT`.
    Efault,
    /// Another errno returned by uname compatibility handling.
    Other(Errno),
}

#[cfg(target_arch = "x86")]
impl OldoldunameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Errors returned by [`olduname`].
#[cfg(target_arch = "x86")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OldunameError {
    /// `EFAULT`.
    Efault,
    /// Another errno returned by uname compatibility handling.
    Other(Errno),
}

#[cfg(target_arch = "x86")]
impl OldunameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Errors returned by [`newuname`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewunameError {
    /// `EFAULT`.
    Efault,
    /// Another errno returned by uname compatibility handling.
    Other(Errno),
}

impl NewunameError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Copy system identity strings through the legacy x86 `oldolduname` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldOldUtsname>` and maps the raw syscall return into
/// `Result<(), OldoldunameError>`.
///
/// On success, the kernel has initialized `name` with the five-field legacy
/// `OldOldUtsname` record.
///
/// See [`sys::oldolduname`] for kernel behavior, reachable errors, ABI layout,
/// and source references.
///
/// # Errors
/// - [`OldoldunameError::Efault`]: the kernel could not write the output
///   buffer.
/// - [`OldoldunameError::Other`]: another errno from uname compatibility
///   handling.
#[cfg(target_arch = "x86")]
pub fn oldolduname(
    name: &mut MaybeUninit<OldOldUtsname>,
) -> Result<(), OldoldunameError> {
    // SAFETY: `MaybeUninit<OldOldUtsname>` provides writable storage for one
    // kernel-initialized legacy uname record.
    let ret = unsafe { sys::oldolduname(name.as_mut_ptr()) };

    unit_from_ret(ret as isize, OldoldunameError::from_errno)
}

/// Copy system identity strings through the legacy x86 `olduname` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldUtsname>` and maps the raw syscall return into
/// `Result<(), OldunameError>`.
///
/// On success, the kernel has initialized `name` with the five-field
/// `OldUtsname` record.
///
/// See [`sys::olduname`] for kernel behavior, reachable errors, ABI layout,
/// and source references.
///
/// # Errors
/// - [`OldunameError::Efault`]: the kernel could not write the output buffer.
/// - [`OldunameError::Other`]: another errno from uname compatibility
///   handling.
#[cfg(target_arch = "x86")]
pub fn olduname(
    name: &mut MaybeUninit<OldUtsname>,
) -> Result<(), OldunameError> {
    // SAFETY: `MaybeUninit<OldUtsname>` provides writable storage for one
    // kernel-initialized legacy uname record.
    let ret = unsafe { sys::olduname(name.as_mut_ptr()) };

    unit_from_ret(ret as isize, OldunameError::from_errno)
}

/// Copy the system identity strings into `name`.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<NewUtsname>` and maps the raw syscall return into
/// `Result<(), NewunameError>`.
///
/// On success, the kernel has initialized `name` with the six-field
/// `NewUtsname` record.
///
/// See [`sys::newuname`] for kernel behavior, reachable errors, ABI layout,
/// and source references.
///
/// # Errors
/// - [`NewunameError::Efault`]: the kernel could not write the output buffer.
/// - [`NewunameError::Other`]: another errno from uname compatibility handling.
pub fn newuname(
    name: &mut MaybeUninit<NewUtsname>,
) -> Result<(), NewunameError> {
    // SAFETY: `MaybeUninit<NewUtsname>` provides writable storage for one
    // kernel-initialized uname record.
    let ret = unsafe { sys::newuname(name.as_mut_ptr()) };

    unit_from_ret(ret as isize, NewunameError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::mem::MaybeUninit;

    use crate::Errno;

    use super::{NewunameError, newuname};
    #[cfg(target_arch = "x86")]
    use super::{OldoldunameError, OldunameError, oldolduname, olduname};

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldolduname_ok() {
        let mut name = MaybeUninit::uninit();

        assert_eq!(oldolduname(&mut name), Ok(()));
        let name = unsafe { name.assume_init() };
        let sysname = name
            .sysname
            .iter()
            .take(5)
            .map(|byte| byte.to_ne_bytes()[0])
            .collect::<Vec<_>>();
        assert_eq!(sysname, b"Linux");
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldolduname_error_mapping() {
        assert_eq!(
            OldoldunameError::from_errno(Errno::Efault),
            OldoldunameError::Efault
        );
        assert_eq!(
            OldoldunameError::from_errno(Errno::Eio),
            OldoldunameError::Other(Errno::Eio)
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_olduname_ok() {
        let mut name = MaybeUninit::uninit();

        assert_eq!(olduname(&mut name), Ok(()));
        let name = unsafe { name.assume_init() };
        assert_ne!(name.sysname[0], 0);
        assert_ne!(name.nodename[0], 0);
        assert_ne!(name.release[0], 0);
        assert_ne!(name.version[0], 0);
        assert_ne!(name.machine[0], 0);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_olduname_error_mapping() {
        assert_eq!(
            OldunameError::from_errno(Errno::Efault),
            OldunameError::Efault
        );
        assert_eq!(
            OldunameError::from_errno(Errno::Eio),
            OldunameError::Other(Errno::Eio)
        );
    }

    #[test]
    fn test_newuname_ok() {
        let mut name = MaybeUninit::uninit();

        assert_eq!(newuname(&mut name), Ok(()));
        let name = unsafe { name.assume_init() };
        let sysname = name
            .sysname
            .iter()
            .take(5)
            .map(|byte| byte.to_ne_bytes()[0])
            .collect::<Vec<_>>();
        assert_eq!(sysname, b"Linux");
    }

    #[test]
    fn test_newuname_error_mapping() {
        assert_eq!(
            NewunameError::from_errno(Errno::Efault),
            NewunameError::Efault
        );
        assert_eq!(
            NewunameError::from_errno(Errno::Eio),
            NewunameError::Other(Errno::Eio)
        );
    }
}
