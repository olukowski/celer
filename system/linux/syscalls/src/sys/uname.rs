use celer_system_linux_ctypes::{Int, NewUtsname};
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::{OldOldUtsname, OldUtsname};

use crate::arch::current::{Sysno, syscall1};

/// Copy the system identity strings through the legacy i386 `oldolduname` ABI
/// into the caller-provided buffer.
///
/// # Safety
/// - If `name` is non-null, it must point to writable memory for one 45-byte
///   legacy record for the duration of the syscall.
/// - `name`, when non-null, must not alias live Rust references or other Rust
///   allocations that the kernel may mutate through this output buffer.
///
/// # Kernel Support
/// - Introduced: Linux 0.10, as syscall number `59` named `uname`
/// - Behavior changes: Linux 1.0's `sys_olduname` entry uses the same legacy
///   45-byte ABI under the `oldolduname` syscall name; newer kernels keep
///   that `oldolduname` ABI.
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
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.10#n216)
#[cfg(target_arch = "x86")]
pub unsafe fn oldolduname(name: *mut OldOldUtsname) -> Int {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall1(Sysno::Oldolduname, name.addr() as isize) }) as Int
}

/// Copy the system identity strings through the i386 `olduname` ABI into the
/// caller-provided buffer.
///
/// # Safety
/// - If `name` is non-null, it must point to writable memory for one
///   `OldUtsname` record for the duration of the syscall.
/// - `name`, when non-null, must not alias live Rust references or other Rust
///   allocations that the kernel may mutate through this output buffer.
///
/// # Kernel Support
/// - Introduced: Linux 0.99.8, as syscall number `109` named `uname`
/// - Behavior changes: Linux 1.0 keeps the same fixed five-field 65-byte
///   string ABI under the `olduname` syscall name. Current kernels may
///   overwrite `release` for `UNAME26` tasks and `machine` for `PER_LINUX32`
///   tasks after the main copy.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - On success, fills `name` with the kernel's five identity strings:
///   `sysname`, `nodename`, `release`, `version`, and `machine`.
/// - The kernel copies a fixed-size `struct old_utsname` record with five
///   65-byte fields.
/// - Linux 1.0 returns `EFAULT` when `name` is null before attempting the
///   copy.
///
/// # Errors
/// - `EFAULT`: `name` is null or not writable for one `OldUtsname` record.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/uname.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v6.19#n1372)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1372)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n604)
/// - First appearance: [Linux 0.99.8](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.99.8#n554)
/// - Linux 1.0 ABI layout:
///   [include/linux/utsname.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/utsname.h?h=1.0#n16)
#[cfg(target_arch = "x86")]
pub unsafe fn olduname(name: *mut OldUtsname) -> Int {
    // SAFETY: the caller guarantees that `name` is a valid writable output
    // buffer with no conflicting Rust aliases for the duration of the
    // syscall.
    (unsafe { syscall1(Sysno::Olduname, name.addr() as isize) }) as Int
}

/// Copy the system identity strings through the i386 `newuname` ABI into the
/// caller-provided buffer.
///
/// # Safety
/// - If `name` is non-null, it must point to writable memory for one
///   `NewUtsname` record for the duration of the syscall.
/// - `name`, when non-null, must not alias live Rust references or other Rust
///   allocations that the kernel may mutate through this output buffer.
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
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys.c?h=v7.0#n1351)
/// - Stable x86_64 table (`uname` -> `sys_newuname`): [v7.0 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n75)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys.c?h=v6.18.18#n1351)
/// - LTS x86_64 table (`uname` -> `sys_newuname`): [v6.18.18 syscall_64.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v6.18.18#n75)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n592)
/// - Linux 1.0 ABI layout:
///   [include/linux/utsname.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/utsname.h?h=1.0#n24)
pub unsafe fn newuname(name: *mut NewUtsname) -> Int {
    // SAFETY: the caller guarantees that `name` is a valid writable output
    // buffer with no conflicting Rust aliases for the duration of the
    // syscall.
    (unsafe { syscall1(Sysno::Newuname, name.addr() as isize) }) as Int
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::NewUtsname;
    #[cfg(target_arch = "x86")]
    use celer_system_linux_ctypes::{OldOldUtsname, OldUtsname};

    use crate::arch::current::Sysno;

    use super::newuname;
    #[cfg(target_arch = "x86")]
    use super::{oldolduname, olduname};

    #[cfg(target_arch = "x86")]
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

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_uname_null_pointer() {
        let ret = unsafe { oldolduname(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }

    #[test]
    fn test_newuname_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Olduname as isize, 109);
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::Newuname as isize, 122);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::Newuname as isize, 160);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(Sysno::Newuname as isize, 63);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_olduname() {
        assert_eq!(core::mem::size_of::<OldUtsname>(), 325);

        let mut name = OldUtsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
        };

        let ret = unsafe { olduname(&raw mut name) };

        assert_eq!(ret, 0, "olduname failed: {ret}");
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
        ] {
            assert!(
                field.contains(&0),
                "each olduname field should contain a trailing NUL"
            );
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_olduname_null_pointer() {
        let ret = unsafe { olduname(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
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

        let ret = unsafe { newuname(&raw mut name) };

        assert_eq!(ret, 0, "newuname failed: {ret}");
        assert_ne!(name.sysname[0], 0);
        assert_ne!(name.nodename[0], 0);
        assert_ne!(name.release[0], 0);
        assert_ne!(name.version[0], 0);
        assert_ne!(name.machine[0], 0);
        assert_ne!(name.domainname[0], 0);

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
        let ret = unsafe { newuname(core::ptr::null_mut()) };

        assert_eq!(ret, -14, "expected EFAULT from null pointer, got {ret}");
    }
}
