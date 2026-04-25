use core::{ffi::CStr, mem::MaybeUninit};

use celer_system_linux_ctypes::Statfs;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Statfs as Linux10Statfs;

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`statfs`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatfsError {
    Efault,
    Enosys,
    Eoverflow,
    Other(Errno),
}

impl StatfsError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Enosys => Self::Enosys,
            Errno::Eoverflow => Self::Eoverflow,
            errno => Self::Other(errno),
        }
    }
}

/// Return filesystem status for the filesystem containing `path`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer with `&mut MaybeUninit<Statfs>`, and maps the raw return into
/// `Result<(), StatfsError>`.
///
/// On success, the kernel has initialized `buf` with filesystem status.
///
/// See [`sys::statfs`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`StatfsError::Efault`]: the kernel could not read the path or write the
///   output buffer.
/// - [`StatfsError::Enosys`]: the resolved filesystem does not implement a
///   `statfs` hook.
/// - [`StatfsError::Eoverflow`]: filesystem statistics could not be
///   represented in the current ABI.
/// - [`StatfsError::Other`]: delegated pathname lookup or filesystem error.
pub fn statfs(
    path: &CStr,
    buf: &mut MaybeUninit<Statfs>,
) -> Result<(), StatfsError> {
    // SAFETY: `CStr` is a readable NUL-terminated path and
    // `MaybeUninit<Statfs>` provides writable storage for one result.
    let ret = unsafe { sys::statfs(path.as_ptr(), buf.as_mut_ptr()) };
    unit_from_ret(ret as isize, StatfsError::from_errno)
}

/// Return filesystem status through the Linux 1.0 `sys_statfs` ABI.
///
/// This safe wrapper mirrors [`statfs`], but uses the Linux 1.0 statfs layout
/// exposed from [`crate::linux_1_0`].
///
/// See [`sys::linux_1_0::statfs`] for historical kernel behavior and source
/// references.
#[cfg(target_arch = "x86")]
pub fn statfs_1_0(
    path: &CStr,
    buf: &mut MaybeUninit<Linux10Statfs>,
) -> Result<(), StatfsError> {
    // SAFETY: `CStr` is a readable NUL-terminated path and
    // `MaybeUninit<Linux10Statfs>` provides writable storage for one result.
    let ret =
        unsafe { sys::linux_1_0::statfs(path.as_ptr(), buf.as_mut_ptr()) };
    unit_from_ret(ret as isize, StatfsError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;
    use std::ffi::CString;

    #[cfg(target_arch = "x86")]
    use super::statfs_1_0;
    use super::{StatfsError, statfs};
    use crate::Errno;

    #[test]
    fn test_statfs_ok() {
        let path = CString::new("/").unwrap();
        let mut buf = MaybeUninit::uninit();

        assert_eq!(statfs(path.as_c_str(), &mut buf), Ok(()));
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_statfs_ok() {
        let path = CString::new("/").unwrap();
        let mut buf = MaybeUninit::uninit();

        assert_eq!(statfs_1_0(path.as_c_str(), &mut buf), Ok(()));
    }

    #[test]
    fn test_statfs_missing_path() {
        let path = CString::new("/celer-statfs-definitely-missing").unwrap();
        let mut buf = MaybeUninit::uninit();

        assert_eq!(
            statfs(path.as_c_str(), &mut buf),
            Err(StatfsError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_statfs_error_mapping() {
        assert_eq!(StatfsError::from_errno(Errno::Efault), StatfsError::Efault);
        assert_eq!(StatfsError::from_errno(Errno::Enosys), StatfsError::Enosys);
        assert_eq!(
            StatfsError::from_errno(Errno::Eoverflow),
            StatfsError::Eoverflow
        );
        assert_eq!(
            StatfsError::from_errno(Errno::Enoent),
            StatfsError::Other(Errno::Enoent)
        );
    }
}
