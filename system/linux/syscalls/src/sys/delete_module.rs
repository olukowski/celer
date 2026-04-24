use celer_system_linux_ctypes::{Char, Int, UnsignedInt};

use crate::arch::current::{Sysno, syscall2};

/// Unload a kernel module by name.
///
/// This wrapper spans the original Linux 1.0 x86 syscall slot `129` ABI and
/// the current native `delete_module(2)` entrypoints exported by this crate on
/// x86 and aarch64. Linux 1.0 reads only `module_name`; current kernels add
/// `flags`, which Linux 1.0 ignores.
///
/// # Safety
/// - When non-null, `module_name` must be valid to read a NUL-terminated string
///   for the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepts a null `module_name` and then only
///   tries to free modules already marked for deletion; current kernels treat
///   the second argument as unload flags, reject unreadable names with
///   `EFAULT`, and perform additional dependency, state, and capability checks
///   before stopping the module.
/// - Availability: Linux 1.0 provides the x86 syscall; current x86 and
///   aarch64 kernels expose the syscall slot, and `CONFIG_MODULES` controls
///   whether that slot reaches the real implementation or the generic
///   `sys_ni` fallback.
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
/// - Current kernels require `CAP_SYS_MODULE` and also reject the syscall when
///   module loading is disabled globally.
///
/// # Behavior
/// - Linux 1.0 copies `module_name` into a fixed-size `MOD_MAX_NAME` buffer.
/// - Linux 1.0 accepts a null `module_name`, skips the module lookup, calls
///   `free_modules()`, and returns `0`.
/// - When Linux 1.0 finds the named module and it is still running, it calls
///   the module cleanup function before marking the module deleted.
/// - Linux 1.0 ignores `flags`.
/// - Current kernels interpret `flags` according to the modern
///   `delete_module(2)` ABI.
/// - Current kernels pass `module_name` to `strncpy_from_user`; a null
///   `module_name` is rejected with `EFAULT` after the permission and global
///   module-loading checks pass.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to unload modules.
/// - `E2BIG`: Linux 1.0 copied `MOD_MAX_NAME` bytes without seeing a trailing
///   NUL in `module_name`.
/// - `ENOENT`: Linux 1.0 could not find the named module.
/// - `EFAULT`: current kernels cannot read `module_name` from user memory.
/// - `ENOENT`: current kernels receive an empty or too-long module name, or no
///   loaded module matches the copied name.
/// - `EINTR`: current kernels are interrupted while waiting for
///   `module_mutex`.
/// - `EWOULDBLOCK`: current kernels find that other modules still depend on
///   the target, or cannot stop it without force.
/// - `EBUSY`: current kernels find the module already unloading, not live, or
///   lacking an unload path that force flags can use.
/// - `ENOSYS`: current kernels built without `CONFIG_MODULES` route this
///   syscall slot to the generic `sys_ni` fallback.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/delete_module.2.html)
/// - Stable x86 table:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n144)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n294)
/// - Stable implementation:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/module/main.c?h=v7.0#n776)
/// - Stable fallback:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys_ni.c?h=v7.0#n92)
/// - LTS x86 table:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n144)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n294)
/// - LTS implementation:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/module/main.c?h=v6.18.18#n776)
/// - LTS fallback:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/sys_ni.c?h=v6.18.18#n92)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n110)
pub unsafe fn delete_module(
    module_name: *const Char,
    flags: UnsignedInt,
) -> Int {
    // SAFETY: guaranteed by caller. Linux 1.0 ignores `flags`, while current
    // kernels interpret it as the modern unload-flags argument.
    unsafe {
        syscall2(
            Sysno::DeleteModule,
            module_name.addr() as isize,
            flags as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, Int, UnsignedInt};

    use crate::arch::current::{Sysno, syscall2};

    use super::delete_module;

    #[test]
    fn test_delete_module_syscall_number() {
        #[cfg(target_arch = "x86")]
        let expected = 129;
        #[cfg(target_arch = "aarch64")]
        let expected = 106;

        assert_eq!(Sysno::DeleteModule as isize, expected);
    }

    #[test]
    fn test_delete_module_matches_raw_syscall_for_missing_module() {
        let wrapped =
            // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
            unsafe { delete_module(c"definitely_not_a_loaded_celer_module".as_ptr(), 0) };
        // SAFETY: this uses the same raw pointer and flags as the wrapper.
        let raw = unsafe {
            syscall2(
                Sysno::DeleteModule,
                c"definitely_not_a_loaded_celer_module".as_ptr().addr()
                    as isize,
                0,
            ) as Int
        };

        assert_eq!(
            wrapped, raw,
            "delete_module wrapper should match raw syscall"
        );
        assert!(
            matches!(wrapped, -1 | -2 | -38),
            "expected EPERM, ENOENT, or ENOSYS from delete_module(missing), got {wrapped}"
        );
    }

    #[test]
    fn test_delete_module_long_name_matches_raw_syscall() {
        let name = [b'a'; 64];
        let wrapped =
            // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
            unsafe { delete_module(name.as_ptr().cast::<Char>(), 0 as UnsignedInt) };
        // SAFETY: this uses the same non-NUL-terminated name buffer and flags
        // as the wrapper under test.
        let raw = unsafe {
            syscall2(
                Sysno::DeleteModule,
                name.as_ptr().cast::<Char>().addr() as isize,
                0,
            ) as Int
        };

        assert_eq!(
            wrapped, raw,
            "delete_module wrapper should match raw syscall"
        );
        assert!(
            matches!(wrapped, -1 | -2 | -38),
            "expected EPERM, ENOENT, or ENOSYS from delete_module(long name), got {wrapped}"
        );
    }

    #[test]
    fn test_delete_module_nonzero_flags_match_raw_syscall() {
        let flags = 0x1234_5678_u32 as UnsignedInt;
        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let wrapped = unsafe {
            delete_module(
                c"definitely_not_a_loaded_celer_module".as_ptr(),
                flags,
            )
        };
        // SAFETY: this uses the same raw pointer and nonzero flags as the
        // wrapper under test.
        let raw = unsafe {
            syscall2(
                Sysno::DeleteModule,
                c"definitely_not_a_loaded_celer_module".as_ptr().addr()
                    as isize,
                flags as isize,
            ) as Int
        };

        assert_eq!(
            wrapped, raw,
            "delete_module wrapper should forward nonzero flags"
        );
        assert!(
            matches!(wrapped, -1 | -2 | -38),
            "expected EPERM, ENOENT, or ENOSYS from delete_module(nonzero flags), got {wrapped}"
        );
    }
}
