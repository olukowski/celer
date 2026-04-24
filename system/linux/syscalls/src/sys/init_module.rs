use celer_system_linux_ctypes::{Char, Int, UnsignedLong, Void};
#[cfg(target_arch = "x86")]
use celer_system_linux_ctypes::{ModRoutines, UnsignedInt};

use crate::arch::current::{Sysno, syscall3};
#[cfg(target_arch = "x86")]
use crate::arch::linux_1_0::{
    Sysno as Linux10Sysno, syscall4 as linux_1_0_syscall4,
};

/// Load a kernel module image from user memory.
///
/// This wrapper exposes the current x86 `init_module(2)` ABI at syscall slot
/// `128`. Linux 1.0 used a different four-argument ABI at the same slot:
/// `sys_init_module(char *module_name, char *code, unsigned codesize,
/// struct mod_routines *routines)`.
///
/// # Safety
/// - `umod` must be valid to read `len` bytes for the duration of the
///   syscall.
/// - `uargs` must be valid to read a NUL-terminated string for the duration
///   of the syscall.
///
/// # Kernel Support
/// - Historical slot introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 loaded a named module into an existing
///   kernel-resident module slot and called its `init` routine through the
///   supplied routine table; current x86 kernels instead copy a complete
///   module image from user memory, validate it as a loadable module, and pass
///   `uargs` to the module-parameter parser and init path.
/// - Availability: this wrapper is ABI-correct for current supported x86 Linux
///   kernels; it is not ABI-compatible with Linux 1.0. Current kernels built
///   with `CONFIG_MODULES=n` keep the syscall number wired but route it to
///   `sys_ni_syscall`, which returns `ENOSYS`
///
/// # Required Privileges
/// - Current kernels: the caller must have `CAP_SYS_MODULE`, and module
///   loading must not be globally disabled.
///
/// # Behavior
/// - `umod` points to the module image bytes to load.
/// - `len` is the module image length in bytes.
/// - `uargs` points to a NUL-terminated module-parameter string; use `c""`
///   for no parameters.
/// - Current kernels validate the copied image before making the module live,
///   including ELF/module structure, duplicate-load state, and module
///   signatures when configured.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to load modules, module loading is
///   disabled, or a current-kernel blacklist policy rejects the module.
/// - `ENOENT`: a current kernel cannot resolve a required non-weak symbol.
/// - `EINVAL`: a current kernel rejects malformed namespace/import or argument
///   state after the syscall reaches those checks.
/// - `EBUSY`: a current kernel detects an in-progress duplicate load or
///   still-loading symbol owner.
/// - `EINTR`: a current kernel interrupts the duplicate-load wait with a
///   signal.
/// - `EFAULT`: a current kernel cannot read `umod` or `uargs` from user
///   memory.
/// - `ENOMEM`: a current kernel cannot allocate temporary loader memory.
/// - `ENOEXEC`: a current kernel rejects the copied image as too short or as
///   invalid module/ELF input.
/// - `EEXIST`: a current kernel finds that the same module is already live.
/// - `EBADMSG`: a current kernel rejects a malformed module-signature trailer.
/// - `EKEYREJECTED`: a current kernel enforces module signatures and rejects
///   the supplied module image.
/// - `ENOSYS`: a current kernel is built with `CONFIG_MODULES=n`, so the
///   syscall table entry falls back to `sys_ni_syscall`.
/// - Current kernels may also return additional policy-dependent errors from
///   Linux security hooks; this wrapper does not pin those to a stable set.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/init_module.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/module/main.c?h=v6.19#n3569)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/module/main.c?h=v6.18.18#n3563)
/// - Current x86-32 syscall table: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n143)
///
/// # Historical References
/// - Linux 1.0 implementation with the incompatible 4-argument ABI:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n71)
pub unsafe fn init_module(
    umod: *const Void,
    len: UnsignedLong,
    uargs: *const Char,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::InitModule,
            umod.addr() as isize,
            len as isize,
            uargs.addr() as isize,
        ) as Int
    }
}

/// Initialize a Linux 1.0 loadable-kernel-module allocation.
///
/// This is the historical Linux 1.0 ABI at syscall slot `128`:
/// `sys_init_module(char *module_name, char *code, unsigned codesize,
/// struct mod_routines *routines)`.
///
/// # Safety
/// - `module_name` must point to a readable NUL-terminated user string for
///   the duration of the syscall.
/// - `code` must point to `codesize` readable bytes containing the module
///   image payload to copy into the allocation created by Linux 1.0
///   `create_module`.
/// - `routines` must point to one readable [`ModRoutines`] value for the
///   duration of the syscall.
/// - The routine addresses in `routines` must be valid Linux 1.0
///   kernel-callable module entry points. The kernel calls `init` during the
///   syscall and stores `cleanup` for later module unload.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Availability: correct only for Linux 1.0 x86 kernels; current x86 Linux
///   uses the same syscall number for the incompatible three-argument
///   `init_module(2)` ABI exposed by [`init_module`].
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
///
/// # Behavior
/// - Reclaims modules previously marked for deletion.
/// - Copies `module_name` into a fixed-size kernel buffer and looks up an
///   existing uninitialized module allocation by that name.
/// - Copies one [`ModRoutines`] record from user memory.
/// - Rejects a code image whose size would exceed the allocated module page
///   count.
/// - Copies `codesize` bytes from `code` into the module allocation after the
///   leading use-count word, zero-fills the remaining allocation, stores the
///   cleanup routine, calls the init routine, and marks the module running
///   when init returns `0`.
///
/// # Errors
/// - `EPERM`: the caller is not superuser.
/// - `E2BIG`: `module_name` reaches Linux 1.0's fixed `MOD_MAX_NAME` buffer
///   before its trailing NUL byte.
/// - `ENOENT`: no non-deleted module allocation matches `module_name`.
/// - `EINVAL`: `codesize` is larger than the target module allocation.
/// - `EBUSY`: the module init routine returned nonzero.
///
/// The Linux 1.0 entry path does not contain explicit `EFAULT` conversions for
/// invalid `module_name`, `code`, or `routines` pointers.
///
/// # References
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n70)
/// - Linux 1.0 `struct mod_routines`:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/module.h?h=1.0#n30)
/// - Linux 1.0 syscall table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n137)
#[cfg(target_arch = "x86")]
pub unsafe fn init_module_1_0(
    module_name: *const Char,
    code: *const Void,
    codesize: UnsignedInt,
    routines: *const ModRoutines,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        linux_1_0_syscall4(
            Linux10Sysno::InitModule,
            module_name.addr() as isize,
            code.addr() as isize,
            codesize as isize,
            routines.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, ModRoutines, UnsignedLong, Void};

    use crate::arch::current::{Sysno, syscall3};
    #[cfg(target_arch = "x86")]
    use crate::arch::linux_1_0::{
        Sysno as Linux10Sysno, syscall4 as linux_1_0_syscall4,
    };

    use super::init_module;
    #[cfg(target_arch = "x86")]
    use super::init_module_1_0;

    #[test]
    fn test_init_module_sysno() {
        #[cfg(target_arch = "x86")]
        assert_eq!(Sysno::InitModule as isize, 128);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(Sysno::InitModule as isize, 105);
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_init_module_sysno() {
        assert_eq!(Linux10Sysno::InitModule as isize, 128);
    }

    #[test]
    fn test_mod_routines_layout() {
        #[cfg(target_arch = "x86")]
        let expected = (8, 4, 4);
        #[cfg(target_arch = "aarch64")]
        let expected = (16, 8, 8);

        assert_eq!(core::mem::size_of::<ModRoutines>(), expected.0);
        assert_eq!(core::mem::align_of::<ModRoutines>(), expected.1);
        assert_eq!(core::mem::offset_of!(ModRoutines, init), 0);
        assert_eq!(core::mem::offset_of!(ModRoutines, cleanup), expected.2);
    }

    #[test]
    fn test_init_module_empty_image_matches_raw_syscall() {
        let image: [u8; 0] = [];
        let params = c"";

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let wrapped = unsafe {
            init_module(
                image.as_ptr().cast::<Void>(),
                image.len() as UnsignedLong,
                params.as_ptr().cast::<Char>(),
            )
        };
        let raw = unsafe {
            syscall3(
                Sysno::InitModule,
                image.as_ptr().cast::<Void>().addr() as isize,
                image.len() as isize,
                params.as_ptr().cast::<Char>().addr() as isize,
            )
        } as i32;

        assert_eq!(
            wrapped, raw,
            "init_module wrapper should match raw syscall"
        );
        assert!(
            wrapped == -1 || wrapped == -8 || wrapped == -38,
            "expected EPERM, ENOEXEC, or ENOSYS from init_module(empty image), got {wrapped}",
        );
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn test_linux_1_0_init_module_matches_raw_syscall() {
        let module_name = c"";
        let code: [u8; 0] = [];
        let routines = ModRoutines {
            init: 0,
            cleanup: 0,
        };

        let wrapped = unsafe {
            init_module_1_0(
                module_name.as_ptr().cast::<Char>(),
                code.as_ptr().cast::<Void>(),
                code.len() as _,
                &raw const routines,
            )
        };
        let raw = unsafe {
            linux_1_0_syscall4(
                Linux10Sysno::InitModule,
                module_name.as_ptr().cast::<Char>().addr() as isize,
                code.as_ptr().cast::<Void>().addr() as isize,
                code.len() as isize,
                (&raw const routines).addr() as isize,
            )
        } as i32;

        assert_eq!(
            wrapped, raw,
            "Linux 1.0 init_module wrapper should match raw syscall"
        );
        assert!(
            wrapped == -1 || wrapped == -8 || wrapped == -14 || wrapped == -38,
            "expected EPERM, ENOEXEC, EFAULT, or ENOSYS from Linux 1.0 init_module shape on a current kernel, got {wrapped}",
        );
    }
}
