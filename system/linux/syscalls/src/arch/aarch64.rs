use core::arch::asm;

/// Linux aarch64 syscall numbers used by this crate.
///
/// Verified against Linux v7.0 `arch/arm64/tools/syscall_64.tbl`.
#[repr(isize)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sysno {
    Dup = 23,
    Fcntl = 25,
    Ioctl = 29,
    Mount = 40,
    Statfs = 43,
    Fstatfs = 44,
    Truncate = 45,
    Ftruncate = 46,
    Chdir = 49,
    Fchdir = 50,
    Chroot = 51,
    Fchmod = 52,
    Fchown = 55,
    Close = 57,
    Vhangup = 58,
    Lseek = 62,
    Read = 63,
    Write = 64,
    Sync = 81,
    Fsync = 82,
    Acct = 89,
    Getitimer = 102,
    Setitimer = 103,
    InitModule = 105,
    DeleteModule = 106,
    Syslog = 116,
    Ptrace = 117,
    Kill = 129,
    Setpriority = 140,
    Getpriority = 141,
    Reboot = 142,
    Setregid = 143,
    Setgid = 144,
    Setreuid = 145,
    Setuid = 146,
    Times = 153,
    Setpgid = 154,
    Getpgid = 155,
    Setsid = 157,
    Getgroups = 158,
    Setgroups = 159,
    Newuname = 160,
    Sethostname = 161,
    Setdomainname = 162,
    Getrlimit = 163,
    Setrlimit = 164,
    Getrusage = 165,
    Umask = 166,
    Gettimeofday = 169,
    Settimeofday = 170,
    Adjtimex = 171,
    Getpid = 172,
    Getppid = 173,
    Getuid = 174,
    Geteuid = 175,
    Getgid = 176,
    Getegid = 177,
    Sysinfo = 179,
    Brk = 214,
    Munmap = 215,
    Execve = 221,
    Mmap = 222,
    Swapon = 224,
    Swapoff = 225,
    Wait4 = 260,
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

    // SAFETY: `svc #0` is the aarch64 Linux syscall instruction. All other
    // safety requirements are enforced by the caller.
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            lateout("x0") ret,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
            in("x1") arg2,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
            in("x1") arg2,
            in("x2") arg3,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
            in("x4") arg5,
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
    let mut ret = arg1;

    // SAFETY: see [`syscall0`].
    unsafe {
        asm!(
            "svc #0",
            in("x8") sysno as isize,
            inlateout("x0") ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
            in("x4") arg5,
            in("x5") arg6,
            options(nostack),
        );
    }

    ret
}
