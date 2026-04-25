use core::{ffi::CStr, mem::MaybeUninit};

use celer_system_linux_ctypes::{Char, Int};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`readlink`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadlinkError {
    /// `EINVAL`.
    Einval,
    /// `EFAULT`.
    Efault,
    /// `ENAMETOOLONG`.
    Enametoolong,
    /// `ENOENT`.
    Enoent,
    /// `ENOTDIR`.
    Enotdir,
    /// `EACCES`.
    Eacces,
    /// `ENOMEM`.
    Enomem,
    /// Another errno returned by delegated pathname lookup or filesystem code.
    Other(Errno),
}

impl ReadlinkError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Efault => Self::Efault,
            Errno::Raw(36) => Self::Enametoolong,
            Errno::Enoent => Self::Enoent,
            Errno::Enotdir => Self::Enotdir,
            Errno::Eacces => Self::Eacces,
            Errno::Enomem => Self::Enomem,
            errno => Self::Other(errno),
        }
    }
}

/// Read the stored target bytes of the symbolic link named by `path`.
///
/// This safe wrapper takes a NUL-terminated [`CStr`] pathname, replaces the raw
/// output pointer and size with `&mut [MaybeUninit<u8>]`, and maps the raw
/// syscall return value into `Result<usize, ReadlinkError>`.
///
/// On success, returns the number of bytes initialized in `buf`. The kernel
/// does not append a trailing NUL byte.
///
/// See [`sys::readlink`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`ReadlinkError::Einval`]: `buf` is too large for the raw ABI, the raw
///   size is nonpositive, or the resolved inode cannot be read as a symlink.
/// - [`ReadlinkError::Efault`]: the kernel could not read `path` or write
///   `buf`.
/// - [`ReadlinkError::Enametoolong`]: the pathname is too long.
/// - [`ReadlinkError::Enoent`]: the path is empty or missing.
/// - [`ReadlinkError::Enotdir`]: traversal needed a directory.
/// - [`ReadlinkError::Eacces`]: traversal lacked search permission.
/// - [`ReadlinkError::Enomem`]: the kernel could not allocate pathname storage.
/// - [`ReadlinkError::Other`]: delegated pathname lookup or filesystem error.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn readlink(
    path: &CStr,
    buf: &mut [MaybeUninit<u8>],
) -> Result<usize, ReadlinkError> {
    let bufsiz = Int::try_from(buf.len()).map_err(|_| ReadlinkError::Einval)?;
    // SAFETY: `CStr` provides a readable NUL-terminated pathname, and
    // `MaybeUninit<u8>` has the same layout as the writable byte buffer the
    // kernel initializes.
    let ret = unsafe {
        sys::readlink(path.as_ptr(), buf.as_mut_ptr().cast::<Char>(), bufsiz)
    };

    result_from_ret(ret as isize, |ret| ret as usize, ReadlinkError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        ffi::{CString, OsStr},
        fs,
        mem::MaybeUninit,
        os::unix::ffi::OsStrExt as _,
        os::unix::fs::symlink,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::Errno;

    use super::{ReadlinkError, readlink};

    fn temp_path(prefix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("celer_wrap_{prefix}_{now}"));
        path
    }

    #[test]
    fn test_readlink_ok() {
        let link_path = temp_path("readlink_link");
        let target = b"readlink_target";
        symlink(OsStr::from_bytes(target), &link_path).unwrap();
        let path_cstr =
            CString::new(link_path.as_os_str().as_encoded_bytes()).unwrap();
        let mut buf = [MaybeUninit::uninit(); 64];

        let n = readlink(path_cstr.as_c_str(), &mut buf).unwrap();
        assert_eq!(n, target.len());
        let initialized = unsafe {
            core::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), n)
        };
        assert_eq!(initialized, target);

        fs::remove_file(&link_path).unwrap();
    }

    #[test]
    fn test_readlink_regular_file() {
        let path = temp_path("readlink_regular");
        fs::write(&path, b"not a link").unwrap();
        let path_cstr =
            CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        let mut buf = [MaybeUninit::uninit(); 64];

        assert_eq!(
            readlink(path_cstr.as_c_str(), &mut buf),
            Err(ReadlinkError::Einval)
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_readlink_error_mapping() {
        assert_eq!(
            ReadlinkError::from_errno(Errno::Einval),
            ReadlinkError::Einval
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Efault),
            ReadlinkError::Efault
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Raw(36)),
            ReadlinkError::Enametoolong
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Enoent),
            ReadlinkError::Enoent
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Enotdir),
            ReadlinkError::Enotdir
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Eacces),
            ReadlinkError::Eacces
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Enomem),
            ReadlinkError::Enomem
        );
        assert_eq!(
            ReadlinkError::from_errno(Errno::Eio),
            ReadlinkError::Other(Errno::Eio)
        );
    }
}
