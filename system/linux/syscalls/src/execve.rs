use core::{convert::Infallible, ffi::CStr};

use celer_system_linux_ctypes::Char;

use crate::errno::Errno;
use crate::sys;

/// Errors returned by [`execve`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecveError {
    /// `E2BIG`.
    E2big,
    /// `EAGAIN`.
    Eagain,
    /// `EFAULT`.
    Efault,
    /// `ELOOP`.
    Eloop,
    /// `ENOEXEC`.
    Enoexec,
    /// Another errno returned by delegated lookup, permission, allocation, or
    /// binary-loader work.
    Other(Errno),
}

impl ExecveError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::E2big => Self::E2big,
            Errno::Eagain => Self::Eagain,
            Errno::Efault => Self::Efault,
            Errno::Eloop => Self::Eloop,
            Errno::Enoexec => Self::Enoexec,
            errno => Self::Other(errno),
        }
    }
}

/// Replace the current process image with a new program.
///
/// This wrapper takes a NUL-terminated [`CStr`] for `filename`, maps a null
/// `argv` or `envp` vector to `None`, and converts the raw kernel return into
/// `Result<Infallible, ExecveError>`.
///
/// On success, this wrapper does not return because the current process image
/// is replaced.
///
/// See [`sys::execve`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Safety
/// - Every non-null pointer in `argv` and `envp` must be valid to read a
///   NUL-terminated string for the duration of the syscall.
/// - When `argv` or `envp` is `Some`, the slice must end with a null pointer so
///   the kernel's counted walk stays within the borrowed array.
///
/// # Errors
/// - [`ExecveError::E2big`]: the argument or environment data exceeds the
///   kernel's size limits.
/// - [`ExecveError::Eagain`]: `PF_NPROC_EXCEEDED` is still set and the caller
///   remains over `RLIMIT_NPROC`.
/// - [`ExecveError::Efault`]: `argv`, `envp`, or one of their pointed-to
///   strings is inaccessible.
/// - [`ExecveError::Eloop`]: the interpreter or binfmt rewrite limit was hit.
/// - [`ExecveError::Enoexec`]: no binary handler accepted the target image.
/// - [`ExecveError::Other`]: delegated lookup, permission, allocation, or
///   binary-loader error.
pub unsafe fn execve(
    filename: &CStr,
    argv: Option<&[*const Char]>,
    envp: Option<&[*const Char]>,
) -> Result<Infallible, ExecveError> {
    let argv = argv.map_or(core::ptr::null(), <[*const Char]>::as_ptr);
    let envp = envp.map_or(core::ptr::null(), <[*const Char]>::as_ptr);

    // SAFETY: `filename` is a valid NUL-terminated string, and the caller must
    // uphold the remaining raw pointer preconditions for `argv` and `envp`.
    let ret = unsafe { sys::execve(filename.as_ptr(), argv, envp) };
    Err(ExecveError::from_errno(
        Errno::from_kernel_ret(ret as isize)
            .expect("execve success is unreachable because it does not return"),
    ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, Int, PidT};

    use crate::{
        Errno,
        sys::test_support::{_exit as exit, fork, waitpid},
    };

    use super::{ExecveError, execve};

    #[test]
    fn test_execve_ok() {
        let pid = unsafe { fork() };

        fn use_pid(pid: PidT) {
            if pid == 0 {
                let filename = c"/bin/true";
                let argv: [*const Char; 2] =
                    [filename.as_ptr().cast(), core::ptr::null()];
                let envp: [*const Char; 1] = [core::ptr::null()];

                // SAFETY: the pointer arrays are null-terminated, and each
                // pointed-to string remains valid for the duration of the syscall.
                let ret = unsafe { execve(filename, Some(&argv), Some(&envp)) };

                if ret.is_err() {
                    unsafe { exit(1) };
                }
                unsafe { exit(0) };
            }
        }

        use_pid(pid);

        let mut status: Int = 0;
        let waited = unsafe { waitpid(pid, &mut status, 0) };

        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }

    #[test]
    fn test_execve_missing_path() {
        let filename = c"/definitely/not/a/real/celer-wrap-execve";
        let argv: [*const Char; 2] =
            [filename.as_ptr().cast(), core::ptr::null()];
        let envp: [*const Char; 1] = [core::ptr::null()];

        // SAFETY: the pointer arrays are null-terminated, and each pointed-to
        // string remains valid for the duration of the syscall.
        let ret = unsafe { execve(filename, Some(&argv), Some(&envp)) };

        assert_eq!(ret, Err(ExecveError::Other(Errno::Enoent)));
    }

    #[test]
    fn test_execve_null_argv_envp() {
        let filename = c"/definitely/not/a/real/celer-wrap-execve";

        // SAFETY: `argv` and `envp` are passed as null, which the raw syscall
        // accepts, and `filename` stays valid for the duration of the syscall.
        let ret = unsafe { execve(filename, None, None) };

        assert_eq!(ret, Err(ExecveError::Other(Errno::Enoent)));
    }

    #[test]
    fn test_execve_error_mapping() {
        assert_eq!(ExecveError::from_errno(Errno::E2big), ExecveError::E2big);
        assert_eq!(ExecveError::from_errno(Errno::Eagain), ExecveError::Eagain);
        assert_eq!(ExecveError::from_errno(Errno::Efault), ExecveError::Efault);
        assert_eq!(ExecveError::from_errno(Errno::Eloop), ExecveError::Eloop);
        assert_eq!(
            ExecveError::from_errno(Errno::Enoexec),
            ExecveError::Enoexec
        );
        assert_eq!(
            ExecveError::from_errno(Errno::Enoent),
            ExecveError::Other(Errno::Enoent)
        );
    }
}
