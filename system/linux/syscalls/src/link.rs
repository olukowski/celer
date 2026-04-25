use core::ffi::CStr;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`link`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// `EXDEV`.
    Exdev,
    /// Another errno returned by delegated pathname lookup, hard-link
    /// permission checks, filesystem operations, or security hooks.
    Other(Errno),
}

impl LinkError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Exdev => Self::Exdev,
            errno => Self::Other(errno),
        }
    }
}

/// Create a hard link from `newname` to `oldname`.
///
/// This safe wrapper takes NUL-terminated [`CStr`] pathnames and maps the raw
/// syscall return value into `Result<(), LinkError>`.
///
/// On success, returns `Ok(())` after the kernel has created `newname` as
/// another directory entry for the same inode as `oldname`.
///
/// See [`sys::link`] for kernel behavior, reachable errors, and source
/// references.
///
/// # Errors
/// - [`LinkError::Exdev`]: `oldname` and `newname` resolve on different mounts.
/// - [`LinkError::Other`]: delegated pathname lookup, hard-link permission,
///   filesystem, or security-hook error.
pub fn link(oldname: &CStr, newname: &CStr) -> Result<(), LinkError> {
    // SAFETY: both `CStr` values guarantee valid, NUL-terminated pathname
    // pointers for the duration of the call.
    let ret = unsafe { sys::link(oldname.as_ptr(), newname.as_ptr()) };

    unit_from_ret(ret as isize, LinkError::from_errno)
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

    use crate::Errno;

    use super::{LinkError, link};

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
    fn test_link_ok() {
        let old_path = temp_path("link_old");
        let new_path = temp_path("link_new");
        File::create(&old_path).unwrap();
        let old_c =
            CString::new(old_path.as_os_str().as_encoded_bytes()).unwrap();
        let new_c =
            CString::new(new_path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(link(old_c.as_c_str(), new_c.as_c_str()), Ok(()));

        let old_meta = fs::metadata(&old_path).unwrap();
        let new_meta = fs::metadata(&new_path).unwrap();
        assert_eq!(old_meta.ino(), new_meta.ino());
        assert_eq!(old_meta.dev(), new_meta.dev());

        fs::remove_file(&old_path).unwrap();
        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_link_missing_source() {
        let old_path =
            CString::new("/definitely/not/a/real/celer-link-source").unwrap();
        let new_path = temp_path("link_missing_new");
        let new_c =
            CString::new(new_path.as_os_str().as_encoded_bytes()).unwrap();

        assert_eq!(
            link(old_path.as_c_str(), new_c.as_c_str()),
            Err(LinkError::Other(Errno::Enoent))
        );
    }

    #[test]
    fn test_link_error_mapping() {
        assert_eq!(LinkError::from_errno(Errno::Exdev), LinkError::Exdev);
        assert_eq!(
            LinkError::from_errno(Errno::Enoent),
            LinkError::Other(Errno::Enoent)
        );
    }
}
