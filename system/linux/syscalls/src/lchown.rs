use core::ffi::CStr;

use celer_system_linux_ctypes::{OldGidT, OldUidT};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`lchown16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lchown16Error {
    /// `ENOENT`.
    Enoent,
    /// `EPERM`.
    Eperm,
    /// `EROFS`.
    Erofs,
    /// Another errno returned by delegated pathname lookup, filesystem, or
    /// security-hook ownership-change work.
    Other(Errno),
}

impl Lchown16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Enoent => Self::Enoent,
            Errno::Eperm => Self::Eperm,
            Errno::Erofs => Self::Erofs,
            errno => Self::Other(errno),
        }
    }
}

/// Change the owner and/or group of `filename` through the legacy x86
/// `lchown16` ABI without following a final symlink.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, keeps the raw
/// legacy 16-bit owner and group arguments, and maps the raw syscall return
/// value into `Result<(), Lchown16Error>`.
///
/// Pass `OldUidT::MAX` and/or `OldGidT::MAX` to preserve the existing owner or
/// group respectively. On success, returns `Ok(())` after the kernel has
/// applied the ownership-change request to the path itself rather than the
/// symlink target.
///
/// See [`sys::lchown16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Lchown16Error::Enoent`]: `filename` does not resolve to an existing path
///   component.
/// - [`Lchown16Error::Eperm`]: the caller was not allowed to make the
///   ownership change.
/// - [`Lchown16Error::Erofs`]: the target inode lives on a read-only
///   filesystem.
/// - [`Lchown16Error::Other`]: delegated pathname lookup, filesystem, or
///   security-hook ownership-change error.
pub fn lchown16(
    filename: &CStr,
    user: OldUidT,
    group: OldGidT,
) -> Result<(), Lchown16Error> {
    // SAFETY: `CStr` guarantees a valid, NUL-terminated pathname pointer for
    // the duration of the call.
    let ret = unsafe { sys::lchown16(filename.as_ptr(), user, group) };

    unit_from_ret(ret as isize, Lchown16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::CString,
        fs::{self, File},
        os::unix::fs::MetadataExt as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OldGidT, OldUidT};

    use crate::Errno;

    use super::{Lchown16Error, lchown16};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_lchown16_{now}"));
        path
    }

    #[test]
    fn test_lchown16_ok_with_no_change_sentinels() {
        let path = temp_path();
        File::create(&path).unwrap();
        let path_c = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        let before = fs::metadata(&path).unwrap();

        assert_eq!(
            lchown16(path_c.as_c_str(), OldUidT::MAX, OldGidT::MAX),
            Ok(())
        );

        let after = fs::metadata(&path).unwrap();
        assert_eq!(after.uid(), before.uid());
        assert_eq!(after.gid(), before.gid());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_lchown16_missing_path_returns_enoent() {
        let path =
            CString::new("/definitely/not/a/real/celer-lchown16-path").unwrap();

        assert_eq!(
            lchown16(path.as_c_str(), OldUidT::MAX, OldGidT::MAX),
            Err(Lchown16Error::Enoent)
        );
    }

    #[test]
    fn test_lchown16_error_mapping() {
        assert_eq!(
            Lchown16Error::from_errno(Errno::Enoent),
            Lchown16Error::Enoent
        );
        assert_eq!(
            Lchown16Error::from_errno(Errno::Eperm),
            Lchown16Error::Eperm
        );
        assert_eq!(
            Lchown16Error::from_errno(Errno::Erofs),
            Lchown16Error::Erofs
        );
        assert_eq!(
            Lchown16Error::from_errno(Errno::Einval),
            Lchown16Error::Other(Errno::Einval)
        );
    }
}
