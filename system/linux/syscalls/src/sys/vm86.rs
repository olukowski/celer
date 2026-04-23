use celer_system_linux_ctypes::{Int, Vm86Struct};

use crate::arch::current::{Sysno, syscall1};

/// Enter x86 virtual-8086 mode using the historical Linux syscall slot `113`.
///
/// This wrapper targets the original Linux 1.0 `sys_vm86(struct vm86_struct *)`
/// entry point. Current x86 kernels keep syscall number `113` as `vm86old`,
/// still take a single user pointer, and accept the same leading register and
/// flag fields, but they expand the structure through `cpu_type` and add
/// extra trailing state after that prefix.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 copied only the original prefix through
///   `screen_bitmap`, saved the caller's user pointer, and returned `0` when
///   vm86 execution later exited back through the syscall path; current x86
///   kernels keep syscall slot `113` as `vm86old`, require the expanded
///   `struct vm86_struct` layout through `cpu_type`, reject
///   `VM86_SCREEN_BITMAP`, and return encoded `VM86_*` reasons on vm86 exit
/// - Availability: x86 Linux only; current kernels built without
///   `CONFIG_VM86` return `ENOSYS`
///
/// # Required Privileges
/// - Linux 1.0: no explicit privilege check beyond rejecting nested vm86
///   sessions in the same task.
/// - Current kernels: callers must be permitted to map virtual address `0`;
///   the built-in `mmap_min_addr` path rejects disallowed callers.
///
/// # Safety
/// - `v86` must point to a writable userspace [`Vm86Struct`] that remains
///   valid for the entire vm86 session, not merely for the duration of the
///   syscall entry.
/// - Both Linux 1.0 and current x86 kernels retain the user pointer and later
///   write vm86 exit state back through it.
/// - On current kernels, if deferred vm86-exit writeback faults, the kernel
///   sends `SIGSEGV` instead of returning an errno to the caller.
/// - Entering vm86 mode can transfer control according to the register image
///   stored in `v86`.
///
/// # Behavior
/// - `v86` supplies the initial register image and vm86 control flags.
/// - Linux 1.0 forces the saved null segment slots to `0`, sanitizes
///   `eflags`, records `screen_bitmap`, and then transfers into vm86 mode.
/// - Linux 1.0 ignores the trailing `cpu_type`, `int_revectored`, and
///   `int21_revectored` fields because its original structure ended earlier.
/// - Current x86 kernels treat syscall slot `113` as `vm86old`, require the
///   expanded structure prefix through `cpu_type`, reject
///   `flags & VM86_SCREEN_BITMAP`, and leave `screen_bitmap` unchanged on
///   vm86 exit.
/// - Returns a nonnegative value on success, or a negative errno value on
///   failure.
///
/// # Errors
/// - `EPERM`: Linux 1.0 rejects a nested vm86 call when the task is already
///   in vm86 mode; current kernels also reject callers blocked by low-address
///   mapping policy.
/// - `EFAULT`: current kernels cannot read the supplied [`Vm86Struct`] during
///   syscall entry.
/// - `EINVAL`: current kernels reject `flags & VM86_SCREEN_BITMAP`.
/// - `ENOMEM`: current kernels cannot allocate per-task vm86 bookkeeping.
/// - `ENOSYS`: current kernels were built without `CONFIG_VM86`.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/vm86.2.html)
/// - Stable:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/vm86_32.c?h=v7.0#n170)
/// - LTS:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/vm86_32.c?h=v6.18.18#n170)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n165)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n122)
pub unsafe fn vm86(v86: *mut Vm86Struct) -> Int {
    // SAFETY: the caller guarantees that `v86` is a valid vm86 state buffer
    // for as long as the kernel may retain and write through it.
    unsafe { syscall1(Sysno::Vm86, v86.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{
        Int, RevectoredStruct, Vm86Regs, Vm86Struct,
    };

    use crate::arch::current::{Sysno, syscall1};

    use super::vm86;

    const ENOSYS: Int = -(38 as Int);
    const EPERM: Int = -(1 as Int);
    const EFAULT: Int = -(14 as Int);
    const ENOMEM: Int = -(12 as Int);

    #[test]
    fn test_vm86_sysno() {
        assert_eq!(Sysno::Vm86 as isize, 113);
    }

    #[test]
    fn test_vm86_regs_layout() {
        assert_eq!(core::mem::size_of::<Vm86Regs>(), 84);
        assert_eq!(core::mem::align_of::<Vm86Regs>(), 4);
        assert_eq!(core::mem::offset_of!(Vm86Regs, ebx), 0);
        assert_eq!(core::mem::offset_of!(Vm86Regs, eax), 24);
        assert_eq!(core::mem::offset_of!(Vm86Regs, eip), 48);
        assert_eq!(core::mem::offset_of!(Vm86Regs, cs), 52);
        assert_eq!(core::mem::offset_of!(Vm86Regs, eflags), 56);
        assert_eq!(core::mem::offset_of!(Vm86Regs, esp), 60);
        assert_eq!(core::mem::offset_of!(Vm86Regs, ss), 64);
        assert_eq!(core::mem::offset_of!(Vm86Regs, es), 68);
        assert_eq!(core::mem::offset_of!(Vm86Regs, ds), 72);
        assert_eq!(core::mem::offset_of!(Vm86Regs, fs), 76);
        assert_eq!(core::mem::offset_of!(Vm86Regs, gs), 80);
    }

    #[test]
    fn test_vm86_struct_layout() {
        assert_eq!(core::mem::size_of::<RevectoredStruct>(), 32);
        assert_eq!(core::mem::align_of::<RevectoredStruct>(), 4);
        assert_eq!(core::mem::size_of::<Vm86Struct>(), 160);
        assert_eq!(core::mem::align_of::<Vm86Struct>(), 4);
        assert_eq!(core::mem::offset_of!(Vm86Struct, regs), 0);
        assert_eq!(core::mem::offset_of!(Vm86Struct, flags), 84);
        assert_eq!(core::mem::offset_of!(Vm86Struct, screen_bitmap), 88);
        assert_eq!(core::mem::offset_of!(Vm86Struct, cpu_type), 92);
        assert_eq!(core::mem::offset_of!(Vm86Struct, int_revectored), 96);
        assert_eq!(core::mem::offset_of!(Vm86Struct, int21_revectored), 128);
    }

    #[test]
    fn test_vm86_linux_1_0_prefix_matches_historical_size() {
        assert_eq!(core::mem::offset_of!(Vm86Struct, cpu_type), 92);
    }

    #[test]
    fn test_vm86_invalid_pointer_matches_raw_syscall() {
        let ptr = ptr::without_provenance_mut::<Vm86Struct>(1);

        // SAFETY: this deliberately passes an invalid pointer so the kernel
        // rejects the call before entering vm86 mode.
        let wrapped = unsafe { vm86(ptr) };
        // SAFETY: this is the same deliberately invalid pointer value.
        let raw = unsafe { syscall1(Sysno::Vm86, ptr.addr() as isize) as Int };

        assert_eq!(wrapped, raw, "vm86 wrapper should match raw syscall");
        assert!(
            wrapped == EFAULT
                || wrapped == EPERM
                || wrapped == ENOSYS
                || wrapped == ENOMEM,
            "expected EFAULT, EPERM, ENOSYS, or ENOMEM from vm86(invalid), got {wrapped}",
        );
    }
}
