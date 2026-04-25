use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, Ustat};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`ustat`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UstatError {
    Enosys,
    Einval,
    Efault,
    Other(Errno),
}

impl UstatError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enosys => Self::Enosys,
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Return filesystem status through the legacy `ustat` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Ustat>`.
///
/// `Ok(())` means the kernel initialized `ubuf`.
///
/// See [`sys::ustat`] for kernel behavior, ABI notes, reachable errors, and
/// source references.
///
/// # Errors
/// - [`UstatError::Enosys`]: the running kernel reports this entrypoint as
///   unimplemented.
/// - [`UstatError::Einval`]: `dev` does not select a mounted superblock.
/// - [`UstatError::Efault`]: the kernel could not write `ubuf`.
/// - [`UstatError::Other`]: delegated filesystem error.
pub fn ustat(
    dev: Int,
    ubuf: &mut MaybeUninit<Ustat>,
) -> Result<(), UstatError> {
    // SAFETY: `ubuf` provides writable storage for one `Ustat`.
    let ret = unsafe { sys::ustat(dev, ubuf.as_mut_ptr()) };
    unit_from_ret(ret as isize, UstatError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;
    use std::{fs, os::unix::fs::MetadataExt as _};

    use celer_system_linux_ctypes::{Int, Ustat};

    use crate::Errno;

    use super::{UstatError, ustat};

    #[test]
    fn test_ustat_valid_device() {
        let dev = fs::metadata("/").unwrap().dev() as Int;
        let mut ubuf = MaybeUninit::<Ustat>::uninit();

        assert_eq!(ustat(dev, &mut ubuf), Ok(()));
    }

    #[test]
    fn test_ustat_invalid_device() {
        let mut ubuf = MaybeUninit::<Ustat>::uninit();

        assert_eq!(ustat(-1, &mut ubuf), Err(UstatError::Einval));
    }

    #[test]
    fn test_ustat_error_mapping() {
        assert_eq!(UstatError::from_errno(Errno::Enosys), UstatError::Enosys);
        assert_eq!(UstatError::from_errno(Errno::Einval), UstatError::Einval);
        assert_eq!(UstatError::from_errno(Errno::Efault), UstatError::Efault);
        assert_eq!(
            UstatError::from_errno(Errno::Eio),
            UstatError::Other(Errno::Eio)
        );
    }
}
