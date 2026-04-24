use celer_system_linux_ctypes::{Int, UnsignedInt};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`dup2`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dup2Error {
    /// `EBADF`.
    Ebadf,
    /// `EBUSY`.
    Ebusy,
    /// Another errno returned by delegated descriptor-table replacement work.
    Other(Errno),
}

impl Dup2Error {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Ebusy => Self::Ebusy,
            errno => Self::Other(errno),
        }
    }
}

/// Duplicate `oldfd` onto the exact descriptor number `newfd`.
///
/// This safe wrapper maps the raw `dup2(2)` return value into
/// `Result<Int, Dup2Error>` while keeping the file-descriptor arguments as the
/// kernel-facing integer type.
///
/// On success, returns `Ok(newfd)`.
///
/// See [`sys::dup2`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`Dup2Error::Ebadf`]: `oldfd` is not open, or `newfd` is outside the
///   syscall's accepted descriptor range.
/// - [`Dup2Error::Ebusy`]: a current kernel lost a descriptor-table race while
///   replacing `newfd`.
/// - [`Dup2Error::Other`]: another descriptor-table replacement error.
pub fn dup2(oldfd: UnsignedInt, newfd: UnsignedInt) -> Result<Int, Dup2Error> {
    let ret = sys::dup2(oldfd, newfd);

    result_from_ret(ret as isize, |ret| ret as Int, Dup2Error::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::UnsignedInt;

    use super::{Dup2Error, dup2};
    use crate::Errno;

    fn create_temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_wrap_dup2_{now}"));

        path
    }

    #[test]
    fn test_dup2_ok() {
        let old_path = create_temp_path();
        let new_path = create_temp_path();
        let old_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&old_path)
            .unwrap();
        let new_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&new_path)
            .unwrap();

        let oldfd = old_file.into_raw_fd();
        let newfd = new_file.into_raw_fd();

        assert_eq!(dup2(oldfd as UnsignedInt, newfd as UnsignedInt), Ok(newfd));

        assert_eq!(crate::sys::close(newfd), 0);
        assert_eq!(crate::sys::close(oldfd), 0);
        fs::remove_file(&old_path).unwrap();
        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_dup2_same_fd_ok() {
        let path = create_temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();

        assert_eq!(dup2(fd as UnsignedInt, fd as UnsignedInt), Ok(fd));

        assert_eq!(crate::sys::close(fd), 0);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_dup2_ebadf() {
        assert_eq!(dup2(!0 as UnsignedInt, 0), Err(Dup2Error::Ebadf));
    }

    #[test]
    fn test_dup2_error_mapping() {
        assert_eq!(Dup2Error::from_errno(Errno::Ebadf), Dup2Error::Ebadf);
        assert_eq!(Dup2Error::from_errno(Errno::Ebusy), Dup2Error::Ebusy);
        assert_eq!(
            Dup2Error::from_errno(Errno::Enomem),
            Dup2Error::Other(Errno::Enomem)
        );
    }
}
