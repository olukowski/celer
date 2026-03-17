use core::arch::asm;

/// Syscall numbers.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    /// <https://man7.org/linux/man-pages/man2/getpid.2.html>
    Getpid = 20,
}

/// Invoke a syscall with `0` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes no arguments.
/// - Any irreversible side effects of the syscall are intended.
pub unsafe fn syscall0(sysno: Sysno) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "int 0x80",
            in("eax") sysno as usize,
            lateout("eax") ret,
            options(nostack, preserves_flags),
        );
    }

    ret
}
