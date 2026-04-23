use celer_system_linux_ctypes::{Char, UnsignedLong};

use crate::arch::current::{Sysno, syscall2};

/// Create an uninitialized loadable-kernel-module allocation.
///
/// This wrapper targets the original Linux 1.0 i386 syscall slot `127` ABI.
/// Linux 1.0 implements `create_module` as a real module-allocation entry.
/// Current x86 kernels keep the historical syscall number reserved, but route
/// it to `sys_ni_syscall`, which returns `ENOSYS`.
///
/// # Safety
/// - `module_name` must be valid to read a NUL-terminated string for the
///   duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 allocates module metadata and vmalloc-backed
///   storage for a new uninitialized module and returns the module image base
///   address; current x86 kernels keep syscall slot `127` unimplemented and
///   return `ENOSYS`
/// - Availability: implemented in Linux 1.0; unimplemented on current x86
///   kernels
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
///
/// # Behavior
/// - `module_name` must point to a readable NUL-terminated user string for
///   the duration of the syscall.
/// - Rejects `size == 0`.
/// - Copies `module_name` into a fixed-size kernel buffer.
/// - Rejects names that do not fit within Linux 1.0's `MOD_MAX_NAME` limit of
///   `64` bytes including the trailing NUL.
/// - Rejects names already present in the live module list.
/// - Rounds the requested module storage up to whole pages after reserving one
///   leading `int` word for the module use count.
/// - On success, inserts a `MOD_UNINITIALIZED` module into the global module
///   list, stores `0` in the first word of the module image, and returns the
///   base address of that allocation as the raw syscall return value.
/// - The raw return value is address-valued. Callers that want to interpret
///   errors should cast the return value to `isize` or a signed integer type
///   before checking for negative errno results.
///
/// # Errors
/// - `EPERM`: the caller is not superuser.
/// - `EINVAL`: `module_name` is null or `size` is zero.
/// - `E2BIG`: `module_name` reaches Linux 1.0's fixed `MOD_MAX_NAME` buffer
///   before its trailing NUL byte.
/// - `EEXIST`: a non-deleted module with the same name already exists.
/// - `ENOMEM`: Linux 1.0 cannot allocate the saved module name, the module
///   metadata structure, or the vmalloc-backed module image.
///
/// The Linux 1.0 entry path does not contain an explicit `EFAULT` conversion
/// for invalid `module_name` pointers; this wrapper therefore documents only
/// the errno values directly verified on that path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/create_module.2.html)
/// - Stable i386 syscall table:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n142)
/// - Stable `sys_ni_syscall`:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys_ni.c?h=v7.0#n20)
/// - LTS i386 syscall table:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n142)
/// - LTS `sys_ni_syscall`:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys_ni.c?h=v6.18.18#n20)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n20)
pub unsafe fn create_module(
    module_name: *const Char,
    size: UnsignedLong,
) -> UnsignedLong {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(
            Sysno::CreateModule,
            module_name.addr() as isize,
            size as isize,
        ) as UnsignedLong
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::ffi::CString;

    use celer_system_linux_ctypes::{Char, Long, UnsignedLong};

    use crate::arch::current::{Sysno, syscall2};

    use super::create_module;

    #[test]
    fn test_create_module_sysno() {
        assert_eq!(Sysno::CreateModule as isize, 127);
    }

    #[test]
    fn test_create_module_matches_raw_syscall_on_current_kernel() {
        let name = CString::new("celer_test_module").unwrap();
        let size = 4096 as UnsignedLong;

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let wrapped = unsafe { create_module(name.as_ptr().cast(), size) };
        // SAFETY: this uses the same valid string pointer and size as the
        // wrapper under test.
        let raw = unsafe {
            syscall2(
                Sysno::CreateModule,
                name.as_ptr().cast::<Char>().addr() as isize,
                size as isize,
            ) as UnsignedLong
        };

        assert_eq!(wrapped as Long, raw as Long);
        assert_eq!(wrapped as Long, -(38 as Long));
    }
}
