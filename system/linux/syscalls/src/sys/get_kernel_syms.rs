use celer_system_linux_ctypes::{Int, KernelSym};

use crate::arch::current::{Sysno, syscall1};

/// Copy the historical Linux 1.0 kernel symbol table into a caller buffer, or
/// query the symbol count with a null pointer.
///
/// Linux 1.0 exposed `get_kernel_syms` at syscall number `130`. Current
/// x86-32 kernels still reserve that historical syscall number, but leave it
/// unimplemented so the slot resolves to `sys_ni_syscall`.
///
/// # Safety
/// - If `table` is non-null, it must point to writable memory for the full
///   `KernelSym` array that the kernel will write on success.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 returns the kernel symbol count and, for a
///   non-null pointer, copies that many `KernelSym` records; current x86
///   kernels route the reserved syscall slot to `sys_ni_syscall`
/// - Availability: present on Linux 1.0 x86; current x86-32 kernels reserve
///   syscall number `130` but do not implement the historical behavior; x86-64
///   leaves `get_kernel_syms` unimplemented at a different syscall number
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - If `table` is null, Linux 1.0 writes nothing and returns the number of
///   available kernel symbols.
/// - If `table` is non-null, Linux 1.0 first verifies the entire destination
///   range, then copies exactly one `KernelSym` record per exported symbol and
///   returns the same symbol count.
/// - Each record contains the symbol value in `value` and a fixed-size symbol
///   name in `name`.
/// - Linux 1.0 copies names with `strncpy`, so symbol names longer than 60
///   bytes can be truncated without an added trailing NUL.
/// - Current x86 kernels return `ENOSYS` because the reserved syscall-table
///   slot has no native implementation.
///
/// # Errors
/// - Linux 1.0: `EFAULT` if `table` is non-null and the full output range is
///   not writable in user space.
/// - Current x86 kernels: `ENOSYS`, because the syscall number resolves to
///   `sys_ni_syscall`.
///
/// # References
/// - Linux 1.0 syscall number table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n139)
/// - Linux 1.0 syscall table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=1.0#n145)
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/module.c?h=1.0#n132)
/// - Linux 1.0 ABI layout:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/module.h?h=1.0#n36)
/// - Linux 1.0 user-range validation helper:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/mm.h?h=1.0#n14)
/// - Current x86-32 syscall table:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n145)
/// - Current x86-64 syscall table:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v6.19#n189)
/// - Current syscall table generator fallback:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/scripts/syscalltbl.sh?h=v6.19#n87)
/// - Current `sys_ni_syscall` implementation:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/sys_ni.c?h=v6.19#n20)
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn get_kernel_syms(table: *mut KernelSym) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::GetKernelSyms, table.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, KernelSym};

    use crate::arch::current::{Sysno, syscall1};

    use super::get_kernel_syms;

    const EFAULT: Int = -(14 as Int);
    const ENOSYS: Int = -(38 as Int);

    #[test]
    fn test_get_kernel_syms_sysno() {
        assert_eq!(Sysno::GetKernelSyms as isize, 130);
    }

    #[test]
    fn test_get_kernel_syms_layout() {
        assert_eq!(core::mem::size_of::<KernelSym>(), 64);
        assert_eq!(core::mem::align_of::<KernelSym>(), 4);
        assert_eq!(core::mem::offset_of!(KernelSym, value), 0);
        assert_eq!(core::mem::offset_of!(KernelSym, name), 4);
    }

    #[test]
    fn test_get_kernel_syms_null_matches_raw_syscall() {
        let wrapped = unsafe { get_kernel_syms(core::ptr::null_mut()) };
        let raw = unsafe {
            syscall1(
                Sysno::GetKernelSyms,
                core::ptr::null_mut::<KernelSym>().addr() as isize,
            ) as Int
        };

        assert_eq!(
            wrapped, raw,
            "get_kernel_syms wrapper should match raw syscall"
        );
        if wrapped == ENOSYS {
            return;
        }

        assert!(
            wrapped >= 0,
            "get_kernel_syms returned unexpected error: {wrapped}"
        );

        let count = wrapped as usize;
        let mut table = vec![
            KernelSym {
                value: 0,
                name: [0; 60],
            };
            count
        ];
        let copied = unsafe { get_kernel_syms(table.as_mut_ptr()) };

        assert_eq!(
            copied, wrapped,
            "get_kernel_syms copy path should return the same count"
        );
    }

    #[test]
    fn test_get_kernel_syms_bogus_pointer_matches_runtime() {
        let bogus = core::ptr::without_provenance_mut::<KernelSym>(1);

        let wrapped = unsafe { get_kernel_syms(bogus) };
        let raw = unsafe { syscall1(Sysno::GetKernelSyms, 1) as Int };

        assert_eq!(
            wrapped, raw,
            "get_kernel_syms wrapper should match raw syscall for bogus pointer"
        );
        assert!(
            wrapped == ENOSYS || wrapped == EFAULT,
            "get_kernel_syms should return ENOSYS on unimplemented kernels or EFAULT on implemented ones, got {wrapped}"
        );
    }
}
