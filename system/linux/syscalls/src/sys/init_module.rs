use celer_system_linux_ctypes::{Char, Int, UnsignedLong, Void};

use crate::arch::current::{Sysno, syscall3};

/// Load a kernel module image from user memory.
///
/// This wrapper targets the original Linux 1.0 syscall slot `128`, while
/// exposing the current x86 `init_module(2)` ABI. Linux 1.0 used a different
/// four-argument ABI:
/// `sys_init_module(char *module_name, char *code, unsigned codesize,
/// struct mod_routines *routines)`. Current x86 kernels keep the syscall name
/// and i386 number `128` but instead accept a complete module image, its byte
/// length, and a NUL-terminated module-parameter string.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 loaded a named module into an existing
///   kernel-resident module slot and called its `init` routine through the
///   supplied routine table; current kernels copy a complete module image from
///   user memory, validate it as a loadable module, and pass `uargs` to the
///   module-parameter parser and init path.
/// - Availability: present on supported x86 Linux kernels; current kernels
///   built with `CONFIG_MODULES=n` keep the syscall number wired but route it
///   to `sys_ni_syscall`, which returns `ENOSYS`
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser.
/// - Current kernels: the caller must have `CAP_SYS_MODULE`, and module
///   loading must not be globally disabled.
///
/// # Behavior
/// - `umod` points to the module image bytes to load.
/// - `len` is the module image length in bytes.
/// - `uargs` points to a NUL-terminated module-parameter string; use `c""`
///   for no parameters.
/// - Linux 1.0 used a different ABI and therefore did not accept module bytes
///   and parameter strings in this shape.
/// - Current kernels validate the copied image before making the module live,
///   including ELF/module structure, duplicate-load state, and module
///   signatures when configured.
/// - Returns `0` on success, or a negative errno value on failure.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to load modules, module loading is
///   disabled, or a current-kernel blacklist policy rejects the module.
/// - `E2BIG`: Linux 1.0 rejects module names longer than `MOD_MAX_NAME`.
/// - `ENOENT`: Linux 1.0 cannot find the named module slot, or a current
///   kernel cannot resolve a required non-weak symbol.
/// - `EINVAL`: Linux 1.0 finds that the loaded code does not fit the module
///   slot, or a current kernel rejects malformed namespace/import or argument
///   state after the syscall reaches those checks.
/// - `EBUSY`: Linux 1.0 sees a nonzero module init return, or a current kernel
///   detects an in-progress duplicate load or still-loading symbol owner.
/// - `EINTR`: a current kernel interrupts the duplicate-load wait with a
///   signal.
/// - `EFAULT`: a current kernel cannot read `umod` or `uargs`
///   from user memory.
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
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/module/main.c?h=v7.0#n3570)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/module/main.c?h=v6.18.18#n3563)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n71)
/// - Current x86-32 syscall table: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n143)
pub fn init_module(
    umod: *const Void,
    len: UnsignedLong,
    uargs: *const Char,
) -> Int {
    // SAFETY: this wrapper forwards the raw user pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe {
        syscall3(
            Sysno::InitModule,
            umod.addr() as isize,
            len as isize,
            uargs.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, UnsignedLong, Void};

    use crate::arch::current::{Sysno, syscall3};

    use super::init_module;

    #[test]
    fn test_init_module_sysno() {
        assert_eq!(Sysno::InitModule as isize, 128);
    }

    #[test]
    fn test_init_module_empty_image_matches_raw_syscall() {
        let image: [u8; 0] = [];
        let params = c"";

        let wrapped = init_module(
            image.as_ptr().cast::<Void>(),
            image.len() as UnsignedLong,
            params.as_ptr().cast::<Char>(),
        );
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
}
