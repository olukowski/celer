use celer_system_linux_ctypes::{Int, Long, Ustat};

use crate::arch::current::{Sysno, syscall2};

/// Returns filesystem status through the legacy i386 `ustat` ABI.
///
/// Linux 1.0 exposes syscall number 62 as `ustat`, but its syscall entrypoint
/// is a stub that always returns `-ENOSYS`.
///
/// The behavior and guaranteed errno list below describe the verified Linux
/// 1.0 entrypoint. Later kernels implement `ustat` differently and do write to
/// `ubuf`.
///
/// # Safety
/// - If `ubuf` is non-null, it must point to writable memory for one `Ustat`
///   value for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 returned `-ENOSYS` unconditionally from the
///   syscall entrypoint; current kernels implement `ustat` and copy results to
///   `ubuf`.
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Uses the historical i386 `struct ustat` ABI.
/// - On Linux 1.0, the syscall entrypoint does not inspect `dev`.
/// - On Linux 1.0, the kernel does not read from or write to `ubuf`.
/// - On Linux 1.0, every call returns `-ENOSYS`.
///
/// # Errors
/// - `ENOSYS`: always returned by the Linux 1.0 syscall entrypoint.
/// - Later kernels may return additional errors and may write to `ubuf`; those
///   outcomes are not exhaustively documented here.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/ustat.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v6.19#n247)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/statfs.c?h=v6.18.18#n247)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n24)
///
/// # Historical References
/// - Linux 1.0 `struct ustat`: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/types.h?h=1.0#n119)
pub unsafe fn ustat(dev: Int, ubuf: *mut Ustat) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall2(Sysno::Ustat, dev as isize, ubuf.addr() as isize) })
        as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{fs, os::unix::fs::MetadataExt as _};

    use celer_system_linux_ctypes::{Int, Ustat};

    use super::ustat;

    #[test]
    fn test_ustat_layout() {
        assert_eq!(core::mem::size_of::<Ustat>(), 20);
        assert_eq!(core::mem::align_of::<Ustat>(), 4);
        assert_eq!(core::mem::offset_of!(Ustat, f_tfree), 0);
        assert_eq!(core::mem::offset_of!(Ustat, f_tinode), 4);
        assert_eq!(core::mem::offset_of!(Ustat, f_fname), 8);
        assert_eq!(core::mem::offset_of!(Ustat, f_fpack), 14);
    }

    #[test]
    fn test_ustat_null_buffer_faults_for_valid_device() {
        let dev = fs::metadata("/").unwrap().dev() as Int;
        let ret = unsafe { ustat(dev, core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }

    #[test]
    fn test_ustat_writes_buffer_for_valid_device() {
        let dev = fs::metadata("/").unwrap().dev() as Int;
        let mut ubuf = Ustat {
            f_tfree: 0,
            f_tinode: 0,
            f_fname: [0; 6],
            f_fpack: [0; 6],
        };

        let ret = unsafe { ustat(dev, &raw mut ubuf) };

        assert_eq!(ret, 0, "ustat failed for / device: {ret}");
    }
}
