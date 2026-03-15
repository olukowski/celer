use core::arch::asm;

/// Syscall numbers.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    /// <https://man7.org/linux/man-pages/man2/exit.2.html>
    Exit = 1,
    /// <https://man7.org/linux/man-pages/man2/write.2.html>
    Write = 4,
    /// <https://man7.org/linux/man-pages/man2/close.2.html>
    Close = 6,
    /// <https://man7.org/linux/man-pages/man2/getpid.2.html>
    Getpid = 20,
    /// <https://man7.org/linux/man-pages/man2/kill.2.html>
    Kill = 37,
    /// <https://man7.org/linux/man-pages/man2/mmap.2.html>
    Mmap = 90,
    /// <https://man7.org/linux/man-pages/man2/mremap.2.html>
    Mremap = 163,
    /// <https://www.man7.org/linux/man-pages/man2/mmap2.2.html>
    Mmap2 = 192,
    /// <https://man7.org/linux/man-pages/man2/openat.2.html>
    Openat = 295,
}

/// Invoke a syscall with `0` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes no arguments.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
pub unsafe fn syscall0(sysno: Sysno) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
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

/// Invoke a syscall with `1` argument.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly one argument.
/// - `arg0` is a valid argument for `sysno`. If it encodes a pointer, the
///   pointed-to memory must be valid for the duration of the syscall; see
///   [`core::ptr::read`] and [`core::ptr::write`] for what validity requires
///   for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall (e.g. process termination
///   via [`Sysno::Exit`]) are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them. Note that diverging syscalls (e.g. [`Sysno::Exit`])
/// never return.
pub unsafe fn syscall1(sysno: Sysno, arg0: usize) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    unsafe {
        asm!(
            "int 0x80",
            in("eax") sysno as usize,
            in("ebx") arg0,
            lateout("eax") ret,
            options(nostack, preserves_flags),
        );
    }

    ret
}

/// Invoke a syscall with `2` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly two arguments.
/// - `arg0` and `arg1` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
pub unsafe fn syscall2(sysno: Sysno, arg0: usize, arg1: usize) -> isize {
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "int 0x80",
            in("eax") sysno as isize,
            in("ebx") arg0,
            in("ecx") arg1,
            lateout("eax") ret,
            options(nostack, preserves_flags),
        )
    };

    ret
}

/// Invoke a syscall with `3` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly three arguments.
/// - `arg0` through `arg2` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
pub unsafe fn syscall3(
    sysno: Sysno,
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> isize {
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "int 0x80",
            in("eax") sysno as isize,
            in("ebx") arg0,
            in("ecx") arg1,
            in("edx") arg2,
            lateout("eax") ret,
            options(nostack, preserves_flags),
        )
    };

    ret
}

/// Invoke a syscall with `4` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly four arguments.
/// - `arg0` through `arg3` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
pub unsafe fn syscall4(
    sysno: Sysno,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> isize {
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "push esi",
            "mov esi, {arg3_reg}",
            "int 0x80",
            "pop esi",
            in("eax") sysno as isize,
            in("ebx") arg0,
            in("ecx") arg1,
            in("edx") arg2,
            arg3_reg = in(reg) arg3,
            lateout("eax") ret,
            options(preserves_flags),
        )
    };

    ret
}

/// Invoke a syscall with `5` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly five arguments.
/// - `arg0` through `arg4` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
pub unsafe fn syscall5(
    sysno: Sysno,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> isize {
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "push esi",
            "mov esi, {arg3_reg}",
            "int 0x80",
            "pop esi",
            in("eax") sysno as isize,
            in("ebx") arg0,
            in("ecx") arg1,
            in("edx") arg2,
            arg3_reg = in(reg) arg3,
            in("edi") arg4,
            lateout("eax") ret,
            options(preserves_flags),
        )
    };

    ret
}

/// Invoke a syscall with `6` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes exactly six arguments.
/// - `arg0` through `arg5` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
/// - The syscall is valid to invoke given the current process state (e.g.
///   file descriptors are open, required capabilities are held).
/// - Any irreversible side effects of the syscall are intended.
///
/// The return value is the raw kernel return value. Negative values in the
/// range `[-4095, -1]` indicate errno codes; the caller is responsible for
/// interpreting them.
///
/// Note that six arguments is the maximum for the Linux syscall ABI;
/// no Linux syscall takes more than six arguments.
pub unsafe fn syscall6(
    sysno: Sysno,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "push ebp",
            "push esi",
            "mov esi, {arg3_reg}",
            "mov ebp, {arg5_reg}",
            "int 0x80",
            "pop esi",
            "pop ebp",
            in("eax") sysno as isize,
            in("ebx") arg0,
            in("ecx") arg1,
            in("edx") arg2,
            arg3_reg = in(reg) arg3,
            in("edi") arg4,
            arg5_reg = in(reg) arg5,
            lateout("eax") ret,
            options(preserves_flags),
        )
    };

    ret
}
