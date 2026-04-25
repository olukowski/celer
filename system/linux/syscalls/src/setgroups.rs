use celer_system_linux_ctypes::{Int, OldGidT};

use crate::helpers::unit_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`setgroups16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Setgroups16Error {
    Efault,
    Einval,
    Enomem,
    Enosys,
    Eperm,
    Other(Errno),
}

impl Setgroups16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Einval => Self::Einval,
            Errno::Enomem => Self::Enomem,
            Errno::Enosys => Self::Enosys,
            Errno::Eperm => Self::Eperm,
            other => Self::Other(other),
        }
    }
}

/// Replace supplementary group IDs through the legacy x86 `setgroups16` ABI.
///
/// This safe wrapper accepts a shared slice and passes its length plus element
/// pointer to the raw syscall. An empty slice requests an empty supplementary
/// group set.
///
/// See [`sys::setgroups16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Setgroups16Error::Efault`]: the kernel could not read the group list.
/// - [`Setgroups16Error::Einval`]: the slice is too large for the raw ABI, too
///   many groups were supplied, or a gid is invalid in the current namespace.
/// - [`Setgroups16Error::Enomem`]: the kernel could not allocate credentials.
/// - [`Setgroups16Error::Enosys`]: the legacy ABI is not configured.
/// - [`Setgroups16Error::Eperm`]: the caller lacks permission.
/// - [`Setgroups16Error::Other`]: another errno from credential or security
///   checks.
#[cfg(target_arch = "x86")]
pub fn setgroups16(grouplist: &[OldGidT]) -> Result<(), Setgroups16Error> {
    let len =
        Int::try_from(grouplist.len()).map_err(|_| Setgroups16Error::Einval)?;
    // SAFETY: `grouplist` is readable for `len` entries; empty slices pass a
    // non-null dangling pointer but the kernel does not read it when len is 0.
    let ret = unsafe { sys::setgroups16(len, grouplist.as_ptr()) };
    unit_from_ret(ret as isize, Setgroups16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{Setgroups16Error, setgroups16};

    #[test]
    fn test_setgroups16_empty_slice_fails_or_succeeds_by_permissions() {
        let result = setgroups16(&[]);
        assert!(matches!(
            result,
            Ok(())
                | Err(Setgroups16Error::Eperm)
                | Err(Setgroups16Error::Enosys)
        ));
    }

    #[test]
    fn test_setgroups16_error_mapping() {
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Efault),
            Setgroups16Error::Efault
        );
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Einval),
            Setgroups16Error::Einval
        );
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Enomem),
            Setgroups16Error::Enomem
        );
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Enosys),
            Setgroups16Error::Enosys
        );
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Eperm),
            Setgroups16Error::Eperm
        );
        assert_eq!(
            Setgroups16Error::from_errno(Errno::Eio),
            Setgroups16Error::Other(Errno::Eio)
        );
    }
}
