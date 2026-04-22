use celer_system_linux_ctypes::{Int, OldOldUtsname};

use crate::arch::current::{Sysno, syscall1};

/// Copy the system identity strings into the caller-provided buffer.
///
/// # Safety
/// - If `name` is non-null, it must point to writable memory for one 45-byte
///   legacy record for the duration of the syscall.
///
/// # Kernel Support
/// - Available in Linux 1.0
/// - Behavior changes: Linux 1.0's `sys_olduname` entry uses the legacy
///   `struct oldold_utsname` ABI; newer kernels keep the same ABI under the
///   `oldolduname` syscall name.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `name` with the kernel's five legacy identity strings:
///   `sysname`, `nodename`, `release`, `version`, and `machine`.
/// - Linux 1.0 copies exactly 8 bytes into each field and then writes a
///   trailing NUL byte, producing a 45-byte record.
/// - This wrapper exposes the legacy `oldolduname` ABI, not the later
///   `olduname` or `uname` variants.
///
/// # Errors
/// - `EFAULT`: `name` is null or not writable for the 45-byte legacy record.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/uname.2.html)
/// - Current stable implementation:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1392)
/// - Current stable syscall table:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n74)
/// - Historical Linux 1.0 implementation:
///   [kernel/sys.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n625)
/// - Historical Linux 1.0 syscall table:
///   [kernel/sched.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n131)
pub unsafe fn uname(name: *mut OldOldUtsname) -> Int {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall1(Sysno::Oldolduname, name.addr() as isize) }) as Int
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::OldOldUtsname;

    use super::uname;

    #[test]
    fn test_uname() {
        assert_eq!(core::mem::size_of::<OldOldUtsname>(), 45);

        let mut name = OldOldUtsname {
            sysname: [0; 9],
            nodename: [0; 9],
            release: [0; 9],
            version: [0; 9],
            machine: [0; 9],
        };

        let ret = unsafe { uname(&mut name as *mut OldOldUtsname) };
        assert_eq!(ret, 0, "uname failed: {ret}");

        let expected = *b"Linux";
        for (got, exp) in name.sysname.iter().take(expected.len()).zip(expected)
        {
            assert_eq!(*got as u8, exp);
        }
        assert_eq!(name.sysname[expected.len()], 0);
        assert_eq!(name.sysname[8], 0);
        assert_eq!(name.nodename[8], 0);
        assert_eq!(name.release[8], 0);
        assert_eq!(name.version[8], 0);
        assert_eq!(name.machine[8], 0);
        assert_ne!(name.version[0], 0);
    }

    #[test]
    fn test_uname_null_pointer() {
        let ret = unsafe { uname(core::ptr::null_mut()) };
        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
