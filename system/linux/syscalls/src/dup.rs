use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`dup`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DupError {
    /// `EBADF`.
    Ebadf,
    /// `EMFILE`.
    Emfile,
    /// Another errno returned while expanding the descriptor table.
    Other(Errno),
}

impl DupError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Ebadf => Self::Ebadf,
            Errno::Emfile => Self::Emfile,
            errno => Self::Other(errno),
        }
    }
}

/// Duplicate an open file descriptor.
///
/// This safe wrapper maps the raw `dup(2)` return value into
/// `Result<Int, DupError>` while keeping the file-descriptor argument as the
/// kernel-facing integer type.
///
/// On success, returns the new nonnegative file descriptor selected by the
/// kernel.
///
/// See [`sys::dup`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`DupError::Ebadf`]: `fd` does not name an open file descriptor.
/// - [`DupError::Emfile`]: the calling task cannot allocate another file
///   descriptor slot.
/// - [`DupError::Other`]: another descriptor-table expansion error.
pub fn dup(fd: Int) -> Result<Int, DupError> {
    let ret = sys::dup(fd as _);

    result_from_ret(ret as isize, |ret| ret as Int, DupError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        os::fd::IntoRawFd as _,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{DupError, dup};
    use crate::Errno;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_dup_{now}"));
        path
    }

    #[test]
    fn test_dup_ok() {
        let path = temp_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();

        let fd = file.into_raw_fd();
        let dup_fd = dup(fd).expect("dup should succeed for an open fd");
        assert_ne!(dup_fd, fd);

        assert_eq!(crate::close(dup_fd), Ok(()));
        assert_eq!(crate::close(fd), Ok(()));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_dup_ebadf() {
        assert_eq!(dup(-1), Err(DupError::Ebadf));
    }

    #[test]
    fn test_dup_error_mapping() {
        assert_eq!(DupError::from_errno(Errno::Ebadf), DupError::Ebadf);
        assert_eq!(DupError::from_errno(Errno::Emfile), DupError::Emfile);
        assert_eq!(
            DupError::from_errno(Errno::Enomem),
            DupError::Other(Errno::Enomem)
        );
    }
}
