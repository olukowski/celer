use core::ffi::CStr;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`swapoff`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SwapoffError {
    Eperm,
    Efault,
    Einval,
    Enomem,
    Other(Errno),
}

impl SwapoffError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Eperm => Self::Eperm,
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Disable swapping on an active swap area.
///
/// This safe wrapper takes `specialfile` as a NUL-terminated [`CStr`] pathname
/// and maps the raw return into `Result<(), SwapoffError>`.
///
/// On success, the selected swap area has been disabled.
///
/// See [`sys::swapoff`] for kernel behavior, required privileges, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SwapoffError::Eperm`]: the caller lacks permission to disable swap.
/// - [`SwapoffError::Efault`]: the kernel could not read `specialfile`.
/// - [`SwapoffError::Einval`]: the resolved pathname does not identify an
///   active swap area.
/// - [`SwapoffError::Enomem`]: swap teardown or pathname handling could not
///   allocate memory.
/// - [`SwapoffError::Other`]: delegated pathname lookup, file-open, or swap
///   teardown error.
pub fn swapoff(specialfile: &CStr) -> Result<(), SwapoffError> {
    // SAFETY: `CStr` provides a readable NUL-terminated pathname.
    let ret = unsafe { sys::swapoff(specialfile.as_ptr()) };
    unit_from_ret(ret as isize, SwapoffError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use super::{SwapoffError, swapoff};
    use crate::Errno;

    #[test]
    fn test_swapoff_missing_path_is_permission_or_lookup_error() {
        let path = CString::new("/celer-swapoff-definitely-missing").unwrap();
        let result = swapoff(path.as_c_str());

        assert!(matches!(
            result,
            Err(SwapoffError::Eperm | SwapoffError::Other(Errno::Enoent))
        ));
    }

    #[test]
    fn test_swapoff_error_mapping() {
        assert_eq!(SwapoffError::from_errno(Errno::Eperm), SwapoffError::Eperm);
        assert_eq!(
            SwapoffError::from_errno(Errno::Efault),
            SwapoffError::Efault
        );
        assert_eq!(
            SwapoffError::from_errno(Errno::Einval),
            SwapoffError::Einval
        );
        assert_eq!(
            SwapoffError::from_errno(Errno::Enomem),
            SwapoffError::Enomem
        );
        assert_eq!(
            SwapoffError::from_errno(Errno::Enoent),
            SwapoffError::Other(Errno::Enoent)
        );
    }
}
