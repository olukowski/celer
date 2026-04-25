use core::mem::MaybeUninit;

use celer_system_linux_ctypes::NewUtsname;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

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
