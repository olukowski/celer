use core::mem::MaybeUninit;

use celer_system_linux_ctypes::Sysinfo;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;

use crate::errno::Errno;
use crate::helpers::unit_from_ret;
use crate::sys;

/// Errors returned by [`sysinfo`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SysinfoError {
    Efault,
    Other(Errno),
}

impl SysinfoError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            errno => Self::Other(errno),
        }
    }
}

/// Copy system load, memory, swap, and task summary information into `info`.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Sysinfo>` and maps the raw return into
/// `Result<(), SysinfoError>`.
///
/// On success, the kernel has initialized `info`.
///
/// See [`sys::sysinfo`] for kernel behavior, ABI layout notes, reachable
/// errors, and source references.
///
/// # Errors
/// - [`SysinfoError::Efault`]: the kernel could not write the output record.
/// - [`SysinfoError::Other`]: any other syscall error reported by the raw ABI.
pub fn sysinfo(info: &mut MaybeUninit<Sysinfo>) -> Result<(), SysinfoError> {
    // SAFETY: `info` is writable for one `Sysinfo`; `MaybeUninit` avoids
    // claiming the kernel writes into an already initialized Rust value.
    let ret = unsafe { sys::sysinfo(info.as_mut_ptr()) };
    unit_from_ret(ret as isize, SysinfoError::from_errno)
}

/// Copy Linux 1.0 system summary information into `info`.
///
/// This safe wrapper mirrors [`sys::linux_1_0::sysinfo`] with the historical
/// Linux 1.0 output layout.
///
/// On success, the kernel has initialized `info`.
///
/// See [`sys::linux_1_0::sysinfo`] for kernel behavior, ABI layout notes,
/// reachable errors, and source references.
///
/// # Errors
/// - [`SysinfoError::Efault`]: the kernel could not write the output record.
/// - [`SysinfoError::Other`]: any other syscall error reported by the raw ABI.
#[cfg(target_arch = "x86")]
pub fn sysinfo_1_0(
    info: &mut MaybeUninit<Linux10Sysinfo>,
) -> Result<(), SysinfoError> {
    // SAFETY: `info` is writable for one Linux 1.0 `Sysinfo` record.
    let ret = unsafe { sys::linux_1_0::sysinfo(info.as_mut_ptr()) };
    unit_from_ret(ret as isize, SysinfoError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::Sysinfo;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;

    use crate::Errno;

    #[cfg(target_arch = "x86")]
    use super::sysinfo_1_0;
    use super::{SysinfoError, sysinfo};

    #[test]
    fn test_sysinfo_ok() {
        let mut info = MaybeUninit::<Sysinfo>::uninit();
        sysinfo(&mut info).unwrap();
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_sysinfo_1_0_ok() {
        let mut info = MaybeUninit::<Linux10Sysinfo>::uninit();
        sysinfo_1_0(&mut info).unwrap();
    }

    #[test]
    fn test_sysinfo_error_mapping() {
        assert_eq!(
            SysinfoError::from_errno(Errno::Efault),
            SysinfoError::Efault
        );
        assert_eq!(
            SysinfoError::from_errno(Errno::Eio),
            SysinfoError::Other(Errno::Eio)
        );
    }
}
