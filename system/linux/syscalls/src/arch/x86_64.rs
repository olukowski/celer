use core::arch::asm;

/// Linux x86_64 syscall numbers used by this crate.
///
/// Verified against Linux v7.0 `arch/x86/entry/syscalls/syscall_64.tbl`.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Newstat = 4,
    Newfstat = 5,
    Newlstat = 6,
    Lseek = 8,
    Mmap = 9,
    Munmap = 11,
    Brk = 12,
    Ioctl = 16,
    Access = 21,
    Pipe = 22,
    Select = 23,
    Dup = 32,
    Dup2 = 33,
    Pause = 34,
    Getitimer = 36,
    Alarm = 37,
    Setitimer = 38,
    Getpid = 39,
    Fork = 57,
    Execve = 59,
    Exit = 60,
    Wait4 = 61,
    Kill = 62,
    Newuname = 63,
    Fcntl = 72,
    Fsync = 74,
    Truncate = 76,
    Ftruncate = 77,
    Chdir = 80,
    Fchdir = 81,
    Rename = 82,
    Mkdir = 83,
    Rmdir = 84,
    Creat = 85,
    Link = 86,
    Unlink = 87,
    Symlink = 88,
    Readlink = 89,
    Chmod = 90,
    Fchmod = 91,
    Umask = 95,
    Gettimeofday = 96,
    Getrlimit = 97,
    Getrusage = 98,
    Sysinfo = 99,
    Times = 100,
    Ptrace = 101,
    Syslog = 103,
    Setpgid = 109,
    Getppid = 110,
    Getpgrp = 111,
    Setsid = 112,
    Getgroups = 115,
    Setgroups = 116,
    Getpgid = 121,
    Utime = 132,
    Mknod = 133,
    Ustat = 136,
    Statfs = 137,
    Fstatfs = 138,
    Getpriority = 140,
    Setpriority = 141,
    Vhangup = 153,
    Adjtimex = 159,
    Setrlimit = 160,
    Chroot = 161,
    Sync = 162,
    Acct = 163,
    Settimeofday = 164,
    Mount = 165,
    Umount = 166,
    Swapon = 167,
    Swapoff = 168,
    Reboot = 169,
    Sethostname = 170,
    Setdomainname = 171,
    Iopl = 172,
    Ioperm = 173,
    InitModule = 175,
    DeleteModule = 176,
    Time = 201,
}

/// Invoke a syscall with `0` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes no
/// arguments.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall0(sysno: Sysno) -> isize {
    let ret: isize;

    // SAFETY: `syscall` is the x86_64 Linux syscall instruction. The `rcx`
    // and `r11` registers are clobbered by the instruction itself. All other
    // safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `1` argument.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes one
/// argument and that `arg1` is valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall1(sysno: Sysno, arg1: isize) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `2` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes two
/// arguments and that both arguments are valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall2(sysno: Sysno, arg1: isize, arg2: isize) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `3` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes three
/// arguments and that all arguments are valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall3(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `4` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes four
/// arguments and that all arguments are valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall4(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `5` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes five
/// arguments and that all arguments are valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall5(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}

/// Invoke a syscall with `6` arguments.
///
/// # Safety
///
/// The caller must ensure `sysno` identifies a syscall that takes six
/// arguments and that all arguments are valid for that syscall.
#[cfg_attr(coverage_nightly, coverage(off))]
pub unsafe fn syscall6(
    sysno: Sysno,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
    arg6: isize,
) -> isize {
    let ret: isize;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") sysno as isize => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            in("r9") arg6,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }

    ret
}
