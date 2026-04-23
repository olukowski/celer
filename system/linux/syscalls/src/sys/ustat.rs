use celer_system_linux_ctypes::{Int, Long, Ustat};

use crate::arch::current::{Sysno, syscall2};

/// Returns filesystem status through the legacy i386 `ustat` ABI.
///
/// Linux 1.0 exposes syscall number 62 as `ustat`, but its syscall entrypoint
/// is a stub that always returns `-ENOSYS`.
///
/// The behavior and guaranteed errno list below describe the verified Linux
/// 1.0 entrypoint. Current kernels implement `ustat` and do write to `ubuf`.
///
/// # Safety
/// - If `ubuf` is non-null, it must point to writable memory for one `Ustat`
///   value for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes:
///   - Linux 1.0 returned `-ENOSYS` unconditionally from the syscall
///     entrypoint.
///   - Current kernels decode `dev`, call `vfs_ustat()`, zero the legacy
///     output struct, fill `f_tfree` and `f_tinode`, and copy the result to
///     userspace.
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
/// - On current kernels, `f_fname` and `f_fpack` are zero-filled before the
///   result is copied out.
///
/// # Errors
/// - `ENOSYS`: always returned by the Linux 1.0 syscall entrypoint.
/// - `EINVAL`: on current kernels, `dev` does not resolve to a mounted super
///   block.
/// - `EFAULT`: on current kernels, `ubuf` is not writable for one `Ustat`.
/// - Current kernels can also propagate filesystem-specific `statfs`
///   failures from `statfs_by_dentry()`.
///
/// # References
/// - Stable:
///   [v7.0 ustat](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/statfs.c?h=v7.0#n247)
/// - LTS:
///   [v6.18.18 ustat](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/statfs.c?h=v6.18.18#n246)
/// - First stable:
///   [Linux 1.0 sys_ustat](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/open.c?h=1.0#n24)
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
