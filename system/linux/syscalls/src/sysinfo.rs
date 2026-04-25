use core::mem::MaybeUninit;

use celer_system_linux_ctypes::Sysinfo;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;

use crate::sys;

/// Copy system load, memory, swap, and task summary information into `info`.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<Sysinfo>`.
///
/// On return, the kernel has initialized `info`.
///
/// See [`sys::sysinfo`] for kernel behavior, ABI layout notes, reachable raw
/// errors, and source references.
pub fn sysinfo(info: &mut MaybeUninit<Sysinfo>) {
    // SAFETY: `info` is writable for one `Sysinfo`; `MaybeUninit` avoids
    // claiming the kernel writes into an already initialized Rust value. The
    // only raw error path is an inaccessible output pointer, which this wrapper
    // does not expose.
    let _ = unsafe { sys::sysinfo(info.as_mut_ptr()) };
}

/// Copy Linux 1.0 system summary information into `info`.
///
/// This safe wrapper mirrors [`sys::linux_1_0::sysinfo`] with the historical
/// Linux 1.0 output layout.
///
/// On return, the kernel has initialized `info`.
///
/// See [`sys::linux_1_0::sysinfo`] for kernel behavior, ABI layout notes,
/// reachable raw errors, and source references.
#[cfg(target_arch = "x86")]
pub fn sysinfo_1_0(info: &mut MaybeUninit<Linux10Sysinfo>) {
    // SAFETY: `info` is writable for one Linux 1.0 `Sysinfo` record. The only
    // raw error path is an inaccessible output pointer, which this wrapper
    // does not expose.
    let _ = unsafe { sys::linux_1_0::sysinfo(info.as_mut_ptr()) };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::Sysinfo;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::linux_1_0::Sysinfo as Linux10Sysinfo;

    use super::sysinfo;
    #[cfg(target_arch = "x86")]
    use super::sysinfo_1_0;

    #[test]
    fn test_sysinfo_ok() {
        let mut info = MaybeUninit::<Sysinfo>::uninit();
        sysinfo(&mut info);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_sysinfo_1_0_ok() {
        let mut info = MaybeUninit::<Linux10Sysinfo>::uninit();
        sysinfo_1_0(&mut info);
    }
}
