use core::arch::asm;

/// Syscall numbers.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    /// <https://man7.org/linux/man-pages/man2/write.2.html>
    Write = 1,
    /// <https://man7.org/linux/man-pages/man2/close.2.html>
    Close = 3,
    /// <https://man7.org/linux/man-pages/man2/mmap.2.html>
    Mmap = 9,
    /// <https://man7.org/linux/man-pages/man2/brk.2.html>
    Brk = 12,
    /// <https://man7.org/linux/man-pages/man2/access.2.html>
    Access = 21,
    /// <https://man7.org/linux/man-pages/man2/mremap.2.html>
    Mremap = 25,
    /// <https://man7.org/linux/man-pages/man2/alarm.2.html>
    Alarm = 37,
    /// <https://man7.org/linux/man-pages/man2/getpid.2.html>
    Getpid = 39,
    /// <https://man7.org/linux/man-pages/man2/clone.2.html>
    Clone = 56,
    /// <https://man7.org/linux/man-pages/man2/exit.2.html>
    Exit = 60,
    /// <https://man7.org/linux/man-pages/man2/kill.2.html>
    Kill = 62,
    /// <https://man7.org/linux/man-pages/man2/chdir.2.html>
    Chdir = 80,
    /// <https://man7.org/linux/man-pages/man2/creat.2.html>
    Creat = 85,
    /// <https://man7.org/linux/man-pages/man2/chmod.2.html>
    Chmod = 90,
    /// <https://man7.org/linux/man-pages/man2/adjtimex.2.html>
    Adjtimex = 159,
    /// <https://man7.org/linux/man-pages/man2/chroot.2.html>
    Chroot = 161,
    /// <https://man7.org/linux/man-pages/man2/acct.2.html>
    Acct = 163,
    /// <https://man7.org/linux/man-pages/man2/create_module.2.html>
    CreateModule = 174,
    /// <https://man7.org/linux/man-pages/man2/openat.2.html>
    Openat = 257,
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
    let ret: isize;

    // SAFETY: `syscall0` is the correct x86_64 Linux syscall instruction.
    unsafe {
        asm!(
            "syscall",
            in("rax") sysno as isize,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        )
    };

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
    let ret: isize;

    // SAFETY: See `syscall0`.
    unsafe {
        asm!(
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        )
    };

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
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            in("rsi") arg1,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
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
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
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
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
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
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            in("r8") arg4,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
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
            "syscall",
            in("rax") sysno as isize,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            in("r8") arg4,
            in("r9") arg5,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        )
    };

    ret
}
