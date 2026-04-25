use core::mem::MaybeUninit;

use celer_system_linux_ctypes::OldGidT;

use crate::helpers::result_from_ret;
use crate::{errno::Errno, sys};

/// Errors returned by [`getgroups16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Getgroups16Error {
    Einval,
    Efault,
    Other(Errno),
}

impl From<Errno> for Getgroups16Error {
    fn from(value: Errno) -> Self {
        match value {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            other => Self::Other(other),
        }
    }
}

/// Read the current supplementary group IDs through the legacy x86
/// `getgroups16` ABI.
///
/// This wrapper accepts a mutable output slice and passes its length to the raw
/// syscall. Passing an empty slice performs the raw syscall's count query. On
/// success, it returns the supplementary-group count, which is also the number
/// of group IDs written for non-empty slices.
///
/// See [`sys::getgroups16`] for kernel behavior, historical notes, and source
/// references.
///
/// # Errors
/// - [`Getgroups16Error::Einval`]: the supplied buffer length exceeds the raw
///   ABI's `int` range, or the supplied buffer is smaller than the current
///   supplementary-group count on current x86 kernels.
/// - [`Getgroups16Error::Efault`]: the kernel could not write the group list to
///   the supplied buffer.
/// - [`Getgroups16Error::Other`]: any other syscall error reported by the raw
///   ABI.
pub fn getgroups16(
    grouplist: &mut [MaybeUninit<OldGidT>],
) -> Result<usize, Getgroups16Error> {
    let len =
        i32::try_from(grouplist.len()).map_err(|_| Getgroups16Error::Einval)?;

    // SAFETY: `grouplist` provides writable storage for `len` `OldGidT`
    // elements, and a zero length is a count query where the kernel does not
    // dereference the pointer.
    let ret = unsafe {
        sys::getgroups16(len, grouplist.as_mut_ptr().cast::<OldGidT>())
    };
    result_from_ret(ret as isize, |ret| ret as usize, Getgroups16Error::from)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::OldGidT;

    use crate::sys;

    use super::{Getgroups16Error, getgroups16};

    #[test]
    fn test_getgroups16_matches_raw_for_exact_buffer() {
        let count = unsafe { sys::getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "probe failed: {count}");

        let mut groups = vec![MaybeUninit::<OldGidT>::uninit(); count as usize];
        let wrapped =
            getgroups16(&mut groups).expect("wrapped getgroups16 failed");
        let groups = groups
            .into_iter()
            .map(|group| unsafe { group.assume_init() })
            .collect::<Vec<_>>();

        let mut raw_groups = vec![OldGidT::MAX; count as usize];
        let raw = unsafe { sys::getgroups16(count, raw_groups.as_mut_ptr()) };

        assert_eq!(wrapped, count as usize);
        assert_eq!(raw, count);
        assert_eq!(groups, raw_groups);
    }

    #[test]
    fn test_getgroups16_undersized_buffer_returns_einval() {
        let count = unsafe { sys::getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "probe failed: {count}");
        if count == 0 {
            return;
        }

        let mut groups =
            vec![MaybeUninit::<OldGidT>::uninit(); count as usize - 1];
        let err = getgroups16(&mut groups).unwrap_err();

        assert_eq!(err, Getgroups16Error::Einval);
    }

    #[test]
    fn test_getgroups16_error_mapping() {
        assert_eq!(
            Getgroups16Error::from(crate::Errno::Einval),
            Getgroups16Error::Einval
        );
        assert_eq!(
            Getgroups16Error::from(crate::Errno::Efault),
            Getgroups16Error::Efault
        );
        assert_eq!(
            Getgroups16Error::from(crate::Errno::Enomem),
            Getgroups16Error::Other(crate::Errno::Enomem)
        );
    }
}
