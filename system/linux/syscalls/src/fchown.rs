use celer_system_linux_ctypes::{OldGidT, OldUidT, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`fchown16`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Fchown16Error {
    /// `EBADF`.
    Ebadf,
    /// `ENOENT`.
    Enoent,
    /// `EPERM`.
    Eperm,
    /// `EROFS`.
    Erofs,
    /// Another errno returned by delegated filesystem, security, or remote
    /// ownership-change work.
    Other(Errno),
}

impl Fchown16Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Enoent => Self::Enoent,
            Errno::Eperm => Self::Eperm,
            Errno::Erofs => Self::Erofs,
            errno => Self::Other(errno),
        }
    }
}

/// Change the owner and/or group of an open file descriptor through the
/// legacy i386 `fchown16` ABI.
///
/// This safe wrapper keeps the kernel-facing integer and legacy 16-bit
/// owner/group arguments, but maps the raw syscall return value into
/// `Result<(), Fchown16Error>`.
///
/// Pass `OldUidT::MAX` and/or `OldGidT::MAX` to preserve the existing owner
/// or group respectively. On success, returns `Ok(())` after the kernel has
/// applied the ownership change request.
///
/// See [`sys::fchown16`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Fchown16Error::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`Fchown16Error::Enoent`]: the historical Linux 1.0 file-table entry had
///   no inode attached.
/// - [`Fchown16Error::Eperm`]: the caller was not allowed to make the
///   ownership change.
/// - [`Fchown16Error::Erofs`]: the target inode lives on a read-only
///   filesystem.
/// - [`Fchown16Error::Other`]: delegated filesystem, security, or remote
///   ownership-change error.
pub fn fchown16(
    fd: UnsignedInt,
    user: OldUidT,
    group: OldGidT,
) -> Result<(), Fchown16Error> {
    let ret = sys::fchown16(fd, user, group);

    unit_from_ret(ret as isize, Fchown16Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::AsRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{OldGidT, OldUidT, UnsignedInt};

    use crate::Errno;

    use super::{Fchown16Error, fchown16};

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_fchown16_{now}"));
        path
    }

    #[test]
    fn test_fchown16_ok_with_no_change_sentinels() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        assert_eq!(
            fchown16(
                file.as_raw_fd() as UnsignedInt,
                OldUidT::MAX,
                OldGidT::MAX,
            ),
            Ok(())
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_fchown16_ebadf() {
        assert_eq!(
            fchown16(UnsignedInt::MAX, OldUidT::MAX, OldGidT::MAX),
            Err(Fchown16Error::Ebadf)
        );
    }

    #[test]
    fn test_fchown16_error_mapping() {
        assert_eq!(
            Fchown16Error::from_errno(Errno::Ebadf),
            Fchown16Error::Ebadf
        );
        assert_eq!(
            Fchown16Error::from_errno(Errno::Enoent),
            Fchown16Error::Enoent
        );
        assert_eq!(
            Fchown16Error::from_errno(Errno::Eperm),
            Fchown16Error::Eperm
        );
        assert_eq!(
            Fchown16Error::from_errno(Errno::Erofs),
            Fchown16Error::Erofs
        );
        assert_eq!(
            Fchown16Error::from_errno(Errno::Einval),
            Fchown16Error::Other(Errno::Einval)
        );
    }
}
