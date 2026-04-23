use celer_system_linux_ctypes::{Int, UnsignedLong, Void};

use crate::arch::current::{Sysno, syscall3};

/// Read or update the calling task's x86 local descriptor table state.
///
/// This wrapper targets the original Linux 1.0
/// `sys_modify_ldt(int func, void *ptr, unsigned long bytecount)` entry point.
/// Linux 1.0 accepted only `func == 0` for reads and `func == 1` for writes.
/// Current x86 kernels keep the i386 syscall slot, add `func == 2` and
/// `func == 0x11`, and compile the implementation out behind
/// `CONFIG_MODIFY_LDT_SYSCALL`.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 `func == 0` reads `current->ldt`, or
///   `default_ldt` when no private LDT exists; current x86 stores the LDT in
///   `current->mm->context.ldt`, returns `0` from `func == 0` when no private
///   LDT exists, and reserves `func == 2` for the zero-filled default-LDT
///   read path
/// - Availability: present only on x86; current x86 kernels built without
///   `CONFIG_MODIFY_LDT_SYSCALL` return `ENOSYS`
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `func == 0` reads up to `bytecount` bytes from the calling task's LDT
///   into `ptr`.
/// - Linux 1.0 rejects `func == 0` with `EINVAL` when `ptr` is null.
/// - Linux 1.0 truncates reads to the available LDT bytes and returns the
///   number of bytes copied.
/// - Current x86 `func == 0` returns `0` when no private LDT exists; when an
///   LDT exists but is shorter than `bytecount`, the kernel zero-fills the
///   remaining bytes and returns the requested count.
/// - Linux 1.0 `func == 1` requires `bytecount` to equal
///   `sizeof(struct modify_ldt_ldt_s)`.
/// - Linux 1.0 `func == 1` lazily allocates a private LDT for the task,
///   allows clearing an entry when both `base_addr` and `limit` are zero, and
///   otherwise validates the supplied entry number, contents field, and
///   computed base-plus-limit range before updating the descriptor.
/// - Current x86 kernels keep `func == 1`, add `func == 0x11` for the newer
///   write mode, and interpret the write payload as `struct user_desc`.
/// - Returns `0` on successful writes, the kernel-reported byte count on
///   successful reads (which may also be `0`), or a negative errno value on
///   failure.
///
/// # Safety
/// - When `func == 0` or `func == 2`, `ptr` must be valid for kernel writes
///   of the returned byte count for the duration of the syscall, and those
///   writes must not violate Rust aliasing or lifetime rules.
/// - When `func == 1` or `func == 0x11`, `ptr` must be valid for kernel reads
///   of `bytecount` bytes for the duration of the syscall.
/// - Because the read modes write through `ptr`, this wrapper is `unsafe`
///   even though some `func` values use `ptr` only as input.
///
/// # Errors
/// - `EINVAL`: on Linux 1.0, `func == 0` with a null `ptr`; or `func == 1`
///   with a wrong record size, an out-of-range entry number, `contents == 3`,
///   or a computed segment limit that wraps or reaches kernel space. Current
///   x86 also uses `EINVAL` for invalid write records and for some
///   configuration-dependent segment rejections.
/// - `EFAULT`: the user buffer range named by `ptr` is not accessible for the
///   requested read or write direction.
/// - `ENOMEM`: Linux 1.0 cannot allocate the calling task's private LDT on a
///   write request; current x86 kernels can also report allocation failure on
///   the write path.
/// - `EINTR`: current x86 can interrupt the write path before taking the LDT
///   write semaphore.
/// - `ENOSYS`: Linux 1.0 rejects unsupported `func` values; current x86 also
///   returns it for unsupported `func` values and when
///   `CONFIG_MODIFY_LDT_SYSCALL` is disabled.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/modify_ldt.2.html)
/// - Stable:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/ldt.c?h=v7.0#n667)
/// - LTS:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/ldt.c?h=v7.0#n667)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/ldt.c?h=1.0#n95)
///
/// # Historical References
/// - Linux 1.0 ABI layout:
///   [include/linux/ldt.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/ldt.h?h=1.0#n14)
/// - Linux 1.0 syscall number:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n132)
/// - Current x86-32 syscall table:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n138)
/// - Current x86-64 syscall table:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl?h=v7.0#n166)
pub unsafe fn modify_ldt(
    func: Int,
    ptr: *mut Void,
    bytecount: UnsignedLong,
) -> Int {
    // SAFETY: the caller upholds the pointer validity, aliasing, and lifetime
    // requirements documented above for the selected `func`.
    unsafe {
        syscall3(
            Sysno::ModifyLdt,
            func as isize,
            ptr.addr() as isize,
            bytecount as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Int, UnsignedLong, Void};

    use crate::arch::current::{Sysno, syscall3};

    use super::modify_ldt;

    const ENOSYS: Int = -(38 as Int);
    const EINVAL: Int = -(22 as Int);

    fn raw_modify_ldt(
        func: Int,
        ptr: *mut Void,
        bytecount: UnsignedLong,
    ) -> Int {
        // SAFETY: same raw ABI arguments as the wrapper under test.
        unsafe {
            syscall3(
                Sysno::ModifyLdt,
                func as isize,
                ptr.addr() as isize,
                bytecount as isize,
            ) as Int
        }
    }

    #[test]
    fn test_modify_ldt_sysno() {
        assert_eq!(Sysno::ModifyLdt as isize, 123);
    }

    #[test]
    fn test_modify_ldt_default_read_matches_raw_syscall() {
        let mut buf = [0xA5_u8; 8];
        let wrapped = unsafe {
            modify_ldt(
                2 as Int,
                buf.as_mut_ptr().cast::<Void>(),
                buf.len() as UnsignedLong,
            )
        };
        let mut raw_buf = [0xA5_u8; 8];
        let raw = raw_modify_ldt(
            2 as Int,
            raw_buf.as_mut_ptr().cast::<Void>(),
            raw_buf.len() as UnsignedLong,
        );

        assert_eq!(
            wrapped, raw,
            "modify_ldt wrapper should match raw syscall for func=2"
        );

        if wrapped == ENOSYS {
            return;
        }

        assert_eq!(wrapped, 8, "expected 8-byte default LDT read");
        assert_eq!(buf, [0_u8; 8], "func=2 should zero-fill the buffer");
        assert_eq!(raw_buf, [0_u8; 8], "raw syscall should zero-fill too");
    }

    #[test]
    fn test_modify_ldt_zero_length_default_read_succeeds_or_is_unavailable() {
        let wrapped = unsafe { modify_ldt(2 as Int, core::ptr::null_mut(), 0) };
        let raw = raw_modify_ldt(2 as Int, core::ptr::null_mut(), 0);

        assert_eq!(wrapped, raw, "modify_ldt wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == ENOSYS,
            "expected success or ENOSYS from zero-length default read, got {wrapped}",
        );
    }

    #[test]
    fn test_modify_ldt_write_rejects_wrong_record_size_or_is_unavailable() {
        let wrapped = unsafe { modify_ldt(1 as Int, core::ptr::null_mut(), 0) };
        let raw = raw_modify_ldt(1 as Int, core::ptr::null_mut(), 0);

        assert_eq!(wrapped, raw, "modify_ldt wrapper should match raw syscall");
        assert!(
            wrapped == EINVAL || wrapped == ENOSYS,
            "expected EINVAL or ENOSYS from modify_ldt write with bytecount 0, got {wrapped}",
        );
    }

    #[test]
    fn test_modify_ldt_invalid_func_returns_enosys() {
        let wrapped = unsafe { modify_ldt(3 as Int, core::ptr::null_mut(), 0) };
        let raw = raw_modify_ldt(3 as Int, core::ptr::null_mut(), 0);

        assert_eq!(wrapped, raw, "modify_ldt wrapper should match raw syscall");
        assert_eq!(wrapped, ENOSYS, "expected ENOSYS from unsupported func");
    }

    #[test]
    fn test_modify_ldt_oldmode_clear_entry_succeeds_or_is_unavailable() {
        let payload = [0_u8; 16];
        let wrapped = unsafe {
            modify_ldt(
                1 as Int,
                payload.as_ptr().cast_mut().cast::<Void>(),
                payload.len() as UnsignedLong,
            )
        };
        let raw = raw_modify_ldt(
            1 as Int,
            payload.as_ptr().cast_mut().cast::<Void>(),
            payload.len() as UnsignedLong,
        );

        assert_eq!(wrapped, raw, "modify_ldt wrapper should match raw syscall");
        assert!(
            wrapped == 0 || wrapped == ENOSYS,
            "expected success or ENOSYS from oldmode clear-entry write, got {wrapped}",
        );
    }
}
