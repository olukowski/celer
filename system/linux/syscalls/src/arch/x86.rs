use core::arch::asm;

/// Syscall numbers.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    /// <https://man7.org/linux/man-pages/man2/exit.2.html>
    Exit = 1,
    /// <https://man7.org/linux/man-pages/man2/fork.2.html>
    Fork = 2,
    /// <https://man7.org/linux/man-pages/man2/read.2.html>
    Read = 3,
    /// <https://man7.org/linux/man-pages/man2/write.2.html>
    Write = 4,
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
            inlateout("eax") sysno as usize => ret,
            options(nostack, preserves_flags),
        );
    }

    ret
}

/// Invoke a syscall with `1` argument.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes one argument.
/// - Any irreversible side effects of the syscall are intended.
/// - `arg0` is a valid argument for `sysno`. If it encodes a pointer, the
///   pointed-to memory must be valid for the duration of the syscall; see
///   [`core::ptr::read`] and [`core::ptr::write`] for what validity requires
///   for read-only and write-only pointers respectively.
pub unsafe fn syscall1(sysno: Sysno, arg1: isize) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "int 0x80",
            inlateout("eax") sysno as usize => ret,
            in("ebx") arg1,
            options(nostack, preserves_flags),
        );
    }

    ret
}

/// Invoke a syscall with `3` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes three arguments.
/// - Any irreversible side effects of the syscall are intended.
/// - `arg0` through `arg2` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
pub unsafe fn syscall3(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "int 0x80",
            inlateout("eax") sysno as usize => ret,
            in("ebx") arg1,
            in("ecx") arg2,
            in("edx") arg3,
            options(nostack, preserves_flags),
        );
    }

    ret
}
