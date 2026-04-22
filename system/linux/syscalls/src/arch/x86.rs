use core::arch::asm;

pub mod linux_1_0 {
    use core::arch::asm;

    /// Linux 1.0 x86 syscall numbers used by this crate.
    #[repr(isize)]
    #[non_exhaustive]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum Sysno {
        /// Historical Linux bootstrap syscall used only by init.
        Setup = 0,
    }

    /// Invoke a Linux 1.0 x86 syscall with `1` argument.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `sysno` identifies a Linux 1.0 syscall that takes one argument.
    /// - Any irreversible side effects of the syscall are intended.
    /// - `arg1` is a valid argument for `sysno`. If it encodes a pointer, the
    ///   pointed-to memory must be valid for the duration of the syscall; see
    ///   [`core::ptr::read`] and [`core::ptr::write`] for what validity
    ///   requires for read-only and write-only pointers respectively.
    #[cfg_attr(coverage_nightly, coverage(off))]
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
}

/// Syscall numbers.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    /// <https://man7.org/linux/man-pages/man2/exit.2.html>
    Exit = 1,
    /// <https://man7.org/linux/man-pages/man2/fork.2.html>
    Fork = 2,
    /// <https://man7.org/linux/man-pages/man2/execve.2.html>
    Execve = 11,
    /// <https://man7.org/linux/man-pages/man2/read.2.html>
    Read = 3,
    /// <https://man7.org/linux/man-pages/man2/write.2.html>
    Write = 4,
    /// <https://man7.org/linux/man-pages/man2/open.2.html>
    Open = 5,
    /// <https://man7.org/linux/man-pages/man2/lseek.2.html>
    Lseek = 19,
    /// <https://man7.org/linux/man-pages/man2/mount.2.html>
    Mount = 21,
    /// <https://man7.org/linux/man-pages/man2/chmod.2.html>
    Chmod = 15,
    /// <https://man7.org/linux/man-pages/man2/stat.2.html>
    Stat = 18,
    /// <https://man7.org/linux/man-pages/man2/chdir.2.html>
    Chdir = 12,
    /// <https://man7.org/linux/man-pages/man2/umount.2.html>
    Umount = 22,
    /// <https://man7.org/linux/man-pages/man2/time.2.html>
    Time = 13,
    /// <https://man7.org/linux/man-pages/man2/stime.2.html>
    Stime = 25,
    /// <https://man7.org/linux/man-pages/man2/unlink.2.html>
    Unlink = 10,
    /// <https://man7.org/linux/man-pages/man2/close.2.html>
    Close = 6,
    /// <https://man7.org/linux/man-pages/man2/ioctl.2.html>
    Ioctl = 54,
    /// <https://man7.org/linux/man-pages/man2/fcntl.2.html>
    Fcntl = 55,
    /// <https://man7.org/linux/man-pages/man2/brk.2.html>
    Brk = 45,
    /// <https://man7.org/linux/man-pages/man2/dup.2.html>
    Dup = 41,
    /// <https://man7.org/linux/man-pages/man2/pipe.2.html>
    Pipe = 42,
    /// <https://man7.org/linux/man-pages/man2/times.2.html>
    Times = 43,
    /// <https://man7.org/linux/man-pages/man2/waitpid.2.html>
    Waitpid = 7,
    /// <https://man7.org/linux/man-pages/man2/creat.2.html>
    Creat = 8,
    /// <https://man7.org/linux/man-pages/man2/link.2.html>
    Link = 9,
    /// <https://man7.org/linux/man-pages/man2/setuid.2.html>
    Setuid = 23,
    /// <https://man7.org/linux/man-pages/man2/setpgid.2.html>
    Setpgid = 57,
    /// <https://man7.org/linux/man-pages/man2/mknod.2.html>
    Mknod = 14,
    /// <https://man7.org/linux/man-pages/man2/lchown.2.html>
    Lchown = 16,
    /// <https://man7.org/linux/man-pages/man2/getpid.2.html>
    Getpid = 20,
    /// <https://man7.org/linux/man-pages/man2/gettimeofday.2.html>
    Gettimeofday = 78,
    /// <https://man7.org/linux/man-pages/man2/getuid.2.html>
    Getuid = 24,
    /// <https://man7.org/linux/man-pages/man2/getgid.2.html>
    Getgid = 47,
    /// <https://man7.org/linux/man-pages/man2/geteuid.2.html>
    Geteuid = 49,
    /// <https://man7.org/linux/man-pages/man2/getegid.2.html>
    Getegid = 50,
    /// <https://man7.org/linux/man-pages/man2/acct.2.html>
    Acct = 51,
    /// <https://man7.org/linux/man-pages/man2/ptrace.2.html>
    Ptrace = 26,
    /// <https://man7.org/linux/man-pages/man2/alarm.2.html>
    Alarm = 27,
    /// <https://man7.org/linux/man-pages/man2/fstat.2.html>
    Fstat = 28,
    /// <https://man7.org/linux/man-pages/man2/pause.2.html>
    Pause = 29,
    /// <https://man7.org/linux/man-pages/man2/utime.2.html>
    Utime = 30,
    /// <https://man7.org/linux/man-pages/man2/access.2.html>
    Access = 33,
    /// <https://man7.org/linux/man-pages/man2/nice.2.html>
    Nice = 34,
    /// <https://man7.org/linux/man-pages/man2/sync.2.html>
    Sync = 36,
    /// <https://man7.org/linux/man-pages/man2/kill.2.html>
    Kill = 37,
    /// <https://man7.org/linux/man-pages/man2/signal.2.html>
    Signal = 48,
    /// <https://man7.org/linux/man-pages/man2/rename.2.html>
    Rename = 38,
    /// <https://man7.org/linux/man-pages/man2/mkdir.2.html>
    Mkdir = 39,
    /// <https://man7.org/linux/man-pages/man2/rmdir.2.html>
    Rmdir = 40,
    /// <https://man7.org/linux/man-pages/man2/setgid.2.html>
    Setgid = 46,
    /// <https://man7.org/linux/man-pages/man2/uname.2.html>
    Oldolduname = 59,
    /// <https://man7.org/linux/man-pages/man2/umask.2.html>
    Umask = 60,
    /// <https://man7.org/linux/man-pages/man2/chroot.2.html>
    Chroot = 61,
    /// <https://man7.org/linux/man-pages/man2/ustat.2.html>
    Ustat = 62,
    /// <https://man7.org/linux/man-pages/man2/dup.2.html>
    Dup2 = 63,
    /// <https://man7.org/linux/man-pages/man2/getppid.2.html>
    Getppid = 64,
    /// <https://man7.org/linux/man-pages/man2/getpgrp.2.html>
    Getpgrp = 65,
    /// <https://man7.org/linux/man-pages/man2/setsid.2.html>
    Setsid = 66,
    /// <https://man7.org/linux/man-pages/man2/sigaction.2.html>
    Sigaction = 67,
    /// <https://man7.org/linux/man-pages/man2/sigprocmask.2.html>
    Sgetmask = 68,
    /// <https://man7.org/linux/man-pages/man2/sigprocmask.2.html>
    Ssetmask = 69,
    /// <https://man7.org/linux/man-pages/man2/setreuid.2.html>
    Setreuid = 70,
    /// <https://man7.org/linux/man-pages/man2/setregid.2.html>
    Setregid = 71,
    /// <https://man7.org/linux/man-pages/man2/sigsuspend.2.html>
    Sigsuspend = 72,
    /// <https://man7.org/linux/man-pages/man2/sigpending.2.html>
    Sigpending = 73,
    /// <https://man7.org/linux/man-pages/man2/sethostname.2.html>
    Sethostname = 74,
    /// <https://man7.org/linux/man-pages/man2/getrlimit.2.html>
    Setrlimit = 75,
    /// <https://man7.org/linux/man-pages/man2/getrlimit.2.html>
    Getrlimit = 76,
    /// <https://man7.org/linux/man-pages/man2/getrusage.2.html>
    Getrusage = 77,
}

/// Invoke a syscall with `0` arguments.
///
/// # Safety
///
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn syscall0(sysno: Sysno) -> isize {
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
/// - `arg1` is a valid argument for `sysno`. If it encodes a pointer, the
///   pointed-to memory must be valid for the duration of the syscall; see
///   [`core::ptr::read`] and [`core::ptr::write`] for what validity requires
///   for read-only and write-only pointers respectively.
#[cfg_attr(coverage_nightly, coverage(off))]
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

/// Invoke a syscall with `2` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes two arguments.
/// - Any irreversible side effects of the syscall are intended.
/// - `arg1` and `arg2` are valid arguments for `sysno`. If either encodes a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall2(sysno: Sysno, arg1: isize, arg2: isize) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "int 0x80",
            inlateout("eax") sysno as usize => ret,
            in("ebx") arg1,
            in("ecx") arg2,
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
/// - `arg1` through `arg3` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
#[cfg_attr(coverage_nightly, coverage(off))]
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

/// Invoke a syscall with `4` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes four arguments.
/// - Any irreversible side effects of the syscall are intended.
/// - `arg1` through `arg4` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall4(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // The `esi` register is preserved across the syscall by saving/restoring
    // it because Rust's inline asm does not allow a direct operand constraint
    // for this legacy ABI in the way we need here.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "push esi",
            "mov esi, {arg4_reg}",
            "int 0x80",
            "pop esi",
            inlateout("eax") sysno as usize => ret,
            in("ebx") arg1,
            in("ecx") arg2,
            in("edx") arg3,
            arg4_reg = in(reg) arg4,
            options(preserves_flags),
        );
    }

    ret
}

/// Invoke a syscall with `5` arguments.
///
/// # Safety
///
/// The caller must ensure:
/// - `sysno` identifies a syscall that takes five arguments.
/// - Any irreversible side effects of the syscall are intended.
/// - `arg1` through `arg5` are valid arguments for `sysno`. If any encode a
///   pointer, the pointed-to memory must be valid for the duration of the
///   syscall; see [`core::ptr::read`] and [`core::ptr::write`] for what
///   validity requires for read-only and write-only pointers respectively.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall5(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
) -> isize {
    let mut ret: isize;

    // SAFETY: `int 0x80` is the correct x86 Linux syscall instruction.
    // The `esi` register is preserved across the syscall by saving/restoring
    // it because Rust's inline asm does not allow a direct operand constraint
    // for this legacy ABI in the way we need here.
    // All other safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "push esi",
            "mov esi, {arg4_reg}",
            "int 0x80",
            "pop esi",
            inlateout("eax") sysno as usize => ret,
            in("ebx") arg1,
            in("ecx") arg2,
            in("edx") arg3,
            arg4_reg = in(reg) arg4,
            in("edi") arg5,
            options(preserves_flags),
        );
    }

    ret
}
