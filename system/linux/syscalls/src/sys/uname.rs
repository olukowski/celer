use celer_system_linux_ctypes::{Int, NewUtsname, OldOldUtsname};

use crate::arch::current::{Sysno, syscall1};

/// Copy the system identity strings through the legacy i386 `oldolduname` ABI
/// into the caller-provided buffer.
///
/// # Safety
/// - If `name` is non-null, it must point to writable memory for one 45-byte
///   legacy record for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
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
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1392)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1392)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n625)
pub unsafe fn oldolduname(name: *mut OldOldUtsname) -> Int {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall1(Sysno::Oldolduname, name.addr() as isize) }) as Int
}

/// Copy the system identity strings through the i386 `newuname` ABI into the
/// caller-provided buffer.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 copied the fixed `struct new_utsname`
///   directly; current kernels may overwrite `release` for `UNAME26` tasks and
///   `machine` for `PER_LINUX32` tasks after the main copy.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `name` with the kernel's six identity strings:
///   `sysname`, `nodename`, `release`, `version`, `machine`, and
///   `domainname`.
/// - The kernel copies a fixed-size `struct new_utsname` record with six
///   65-byte fields.
/// - Linux 1.0 returns `EFAULT` when `name` is null before attempting the
///   copy.
///
/// # Errors
/// - `EFAULT`: `name` is null or not writable for one `NewUtsname` record.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/uname.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1351)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1351)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n592)
/// - Linux 1.0 ABI layout:
///   [include/linux/utsname.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/utsname.h?h=1.0#n14)
pub fn newuname(name: *mut NewUtsname) -> Int {
    // SAFETY: this wrapper forwards the raw user pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe { syscall1(Sysno::Newuname, name.addr() as isize) }) as Int
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{NewUtsname, OldOldUtsname};

    use crate::arch::current::Sysno;

    use super::{newuname, oldolduname};

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

        let ret = unsafe { oldolduname(&mut name as *mut OldOldUtsname) };

        assert_eq!(ret, 0, "oldolduname failed: {ret}");

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
        let ret = unsafe { oldolduname(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }

    #[test]
    fn test_newuname_sysno() {
        assert_eq!(Sysno::Newuname as isize, 109);
    }

    #[test]
    fn test_newuname() {
        assert_eq!(core::mem::size_of::<NewUtsname>(), 390);

        let mut name = NewUtsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };

        let ret = newuname(&raw mut name);

        assert_eq!(ret, 0, "newuname failed: {ret}");
        assert_ne!(name.sysname[0], 0);
        assert_ne!(name.nodename[0], 0);
        assert_ne!(name.release[0], 0);
        assert_ne!(name.version[0], 0);
        assert_ne!(name.machine[0], 0);

        for field in [
            &name.sysname,
            &name.nodename,
            &name.release,
            &name.version,
            &name.machine,
            &name.domainname,
        ] {
            assert!(
                field.contains(&0),
                "each newuname field should contain a trailing NUL"
            );
        }
    }

    #[test]
    fn test_newuname_null_pointer() {
        let ret = newuname(core::ptr::null_mut());

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
