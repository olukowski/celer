use core::mem::MaybeUninit;

use celer_system_linux_ctypes::Int;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`pipe`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PipeError {
    /// `EFAULT`.
    Efault,
    /// `EMFILE`.
    Emfile,
    /// `ENFILE`.
    Enfile,
    /// `ENOMEM`.
    Enomem,
    /// Another errno returned by pipe or descriptor allocation.
    Other(Errno),
}

impl PipeError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Emfile => Self::Emfile,
            Errno::Raw(23) => Self::Enfile,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Create a pipe and write the read and write descriptors into `fildes`.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<[Int; 2]>` and maps the raw syscall return into
/// `Result<(), PipeError>`.
///
/// On success, the kernel has initialized `fildes` with `[read_fd, write_fd]`.
///
/// See [`sys::pipe`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`PipeError::Efault`]: the kernel could not write the descriptor array.
/// - [`PipeError::Emfile`]: the caller cannot obtain two more descriptors.
/// - [`PipeError::Enfile`]: system-wide pipe or file allocation failed.
/// - [`PipeError::Enomem`]: the kernel could not allocate pipe metadata.
/// - [`PipeError::Other`]: another pipe or descriptor-allocation error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn pipe(fildes: &mut MaybeUninit<[Int; 2]>) -> Result<(), PipeError> {
    // SAFETY: `MaybeUninit<[Int; 2]>` provides writable storage for the two
    // kernel-initialized pipe descriptors.
    let ret = unsafe { sys::pipe(fildes.as_mut_ptr().cast::<Int>()) };

    unit_from_ret(ret as isize, PipeError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::fs::File;
    use std::mem::MaybeUninit;
    use std::os::fd::FromRawFd as _;

    use crate::Errno;

    use super::{PipeError, pipe};

    #[test]
    fn test_pipe_ok() {
        let mut fds = MaybeUninit::uninit();

        assert_eq!(pipe(&mut fds), Ok(()));
        let [read_fd, write_fd] = unsafe { fds.assume_init() };
        assert_ne!(read_fd, write_fd);

        let read_file = unsafe { File::from_raw_fd(read_fd) };
        let write_file = unsafe { File::from_raw_fd(write_fd) };
        drop(read_file);
        drop(write_file);
    }

    #[test]
    fn test_pipe_error_mapping() {
        assert_eq!(PipeError::from_errno(Errno::Efault), PipeError::Efault);
        assert_eq!(PipeError::from_errno(Errno::Emfile), PipeError::Emfile);
        assert_eq!(PipeError::from_errno(Errno::Raw(23)), PipeError::Enfile);
        assert_eq!(PipeError::from_errno(Errno::Enomem), PipeError::Enomem);
        assert_eq!(
            PipeError::from_errno(Errno::Einval),
            PipeError::Other(Errno::Einval)
        );
    }
}
