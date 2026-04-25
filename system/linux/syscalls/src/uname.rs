use core::mem::MaybeUninit;

use celer_system_linux_ctypes::NewUtsname;
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::{OldOldUtsname, OldUtsname};

use crate::sys;

/// Copy system identity strings through the legacy x86 `oldolduname` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldOldUtsname>`.
///
/// On return, the kernel has initialized `name` with the five-field legacy
/// `OldOldUtsname` record.
///
/// See [`sys::oldolduname`] for kernel behavior, reachable raw errors, ABI
/// layout, and source references.
#[cfg(target_arch = "x86")]
pub fn oldolduname(name: &mut MaybeUninit<OldOldUtsname>) {
    // SAFETY: `MaybeUninit<OldOldUtsname>` provides writable storage for one
    // kernel-initialized legacy uname record. The only raw error path is an
    // inaccessible output pointer, which this wrapper does not expose.
    let _ = unsafe { sys::oldolduname(name.as_mut_ptr()) };
}

/// Copy system identity strings through the legacy x86 `olduname` ABI.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<OldUtsname>`.
///
/// On return, the kernel has initialized `name` with the five-field
/// `OldUtsname` record.
///
/// See [`sys::olduname`] for kernel behavior, reachable raw errors, ABI
/// layout, and source references.
#[cfg(target_arch = "x86")]
pub fn olduname(name: &mut MaybeUninit<OldUtsname>) {
    // SAFETY: `MaybeUninit<OldUtsname>` provides writable storage for one
    // kernel-initialized legacy uname record. The only raw error path is an
    // inaccessible output pointer, which this wrapper does not expose.
    let _ = unsafe { sys::olduname(name.as_mut_ptr()) };
}

/// Copy the system identity strings into `name`.
///
/// This safe wrapper replaces the raw output pointer with
/// `&mut MaybeUninit<NewUtsname>`.
///
/// On return, the kernel has initialized `name` with the six-field
/// `NewUtsname` record.
///
/// See [`sys::newuname`] for kernel behavior, reachable raw errors, ABI
/// layout, and source references.
pub fn newuname(name: &mut MaybeUninit<NewUtsname>) {
    // SAFETY: `MaybeUninit<NewUtsname>` provides writable storage for one
    // kernel-initialized uname record. The only raw error path is an
    // inaccessible output pointer, which this wrapper does not expose.
    let _ = unsafe { sys::newuname(name.as_mut_ptr()) };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::mem::MaybeUninit;

    use super::newuname;
    #[cfg(target_arch = "x86")]
    use super::{oldolduname, olduname};

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_oldolduname_ok() {
        let mut name = MaybeUninit::uninit();

        oldolduname(&mut name);
        let name = unsafe { name.assume_init() };
        let sysname = name
            .sysname
            .iter()
            .take(5)
            .map(|byte| byte.to_ne_bytes()[0])
            .collect::<Vec<_>>();
        assert_eq!(sysname, b"Linux");
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_olduname_ok() {
        let mut name = MaybeUninit::uninit();

        olduname(&mut name);
        let name = unsafe { name.assume_init() };
        assert_ne!(name.sysname[0], 0);
        assert_ne!(name.nodename[0], 0);
        assert_ne!(name.release[0], 0);
        assert_ne!(name.version[0], 0);
        assert_ne!(name.machine[0], 0);
    }

    #[test]
    fn test_newuname_ok() {
        let mut name = MaybeUninit::uninit();

        newuname(&mut name);
        let name = unsafe { name.assume_init() };
        let sysname = name
            .sysname
            .iter()
            .take(5)
            .map(|byte| byte.to_ne_bytes()[0])
            .collect::<Vec<_>>();
        assert_eq!(sysname, b"Linux");
    }
}
