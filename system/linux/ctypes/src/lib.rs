#![no_std]
#![cfg(target_os = "linux")]
#![cfg(target_arch = "x86")]

use core::ffi::{
    c_char, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong,
    c_ulonglong, c_ushort, c_void,
};

/// Equivalent to the `ssize_t` type in C.
pub type SsizeT = isize;

/// Equivalent to the `size_t` type in C.
pub type SizeT = usize;

/// Equivalent to the `char` type in C.
pub type Char = c_char;

/// Equivalent to the `unsigned char` type in C.
pub type UnsignedChar = c_uchar;

/// Equivalent to the `short` type in C.
pub type Short = c_short;

/// Equivalent to the `unsigned short` type in C.
pub type UnsignedShort = c_ushort;

/// Equivalent to the `int` type in C.
pub type Int = c_int;

/// Equivalent to the `unsigned int` type in C.
pub type UnsignedInt = c_uint;

/// Equivalent to the `long` type in C.
pub type Long = c_long;

/// Equivalent to the `unsigned long` type in C.
pub type UnsignedLong = c_ulong;

/// Equivalent to the `long long` type in C.
pub type LongLong = c_longlong;

/// Equivalent to the `unsigned long long` type in C.
pub type UnsignedLongLong = c_ulonglong;

/// Equivalent to the `void` type in C.
pub type Void = c_void;

/// Equivalent to the `umode_t` type in the Linux kernel.
pub type UModeT = UnsignedShort;

/// Equivalent to the `off_t` type in the Linux kernel.
pub type OffT = Long;

/// Equivalent to the `time_t` type in the Linux kernel.
pub type TimeT = Long;

/// Equivalent to the `pid_t` type in the Linux kernel.
pub type PidT = Int;

/// Equivalent to the `uid_t` type in the Linux kernel.
pub type UidT = UnsignedInt;

/// Equivalent to the legacy 16-bit `old_uid_t` type used by i386 compatibility
/// syscall ABIs.
pub type OldUidT = UnsignedShort;

/// Equivalent to the `gid_t` type in the Linux kernel.
pub type GidT = UnsignedInt;

/// Equivalent to the legacy 16-bit `old_gid_t` type used by the i386
/// `setgid` syscall ABI.
pub type OldGidT = UnsignedShort;

/// Equivalent to the legacy `old_sigset_t` type used by the i386
/// `sigaction` syscall ABI.
pub type OldSigsetT = UnsignedLong;

/// Linux `RUSAGE_SELF`.
pub const RUSAGE_SELF: Int = 0;

/// Linux `RUSAGE_CHILDREN`.
pub const RUSAGE_CHILDREN: Int = -1;

/// Linux `PRIO_PROCESS`.
pub const PRIO_PROCESS: Int = 0;

/// Linux `PRIO_PGRP`.
pub const PRIO_PGRP: Int = 1;

/// Linux `PRIO_USER`.
pub const PRIO_USER: Int = 2;

/// Linux `struct __old_kernel_stat` used by the 32-bit `stat` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    pub st_dev: UnsignedShort,
    pub st_ino: UnsignedShort,
    pub st_mode: UnsignedShort,
    pub st_nlink: UnsignedShort,
    pub st_uid: UnsignedShort,
    pub st_gid: UnsignedShort,
    pub st_rdev: UnsignedShort,
    pub st_size: UnsignedLong,
    pub st_atime: UnsignedLong,
    pub st_mtime: UnsignedLong,
    pub st_ctime: UnsignedLong,
}

/// Linux `struct stat` / `struct new_stat` used by the i386 `newstat`,
/// `newlstat`, and `newfstat` syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NewStat {
    pub st_dev: UnsignedShort,
    pub __pad1: UnsignedShort,
    pub st_ino: UnsignedLong,
    pub st_mode: UnsignedShort,
    pub st_nlink: UnsignedShort,
    pub st_uid: UnsignedShort,
    pub st_gid: UnsignedShort,
    pub st_rdev: UnsignedShort,
    pub __pad2: UnsignedShort,
    pub st_size: UnsignedLong,
    pub st_blksize: UnsignedLong,
    pub st_blocks: UnsignedLong,
    pub st_atime: UnsignedLong,
    pub __unused1: UnsignedLong,
    pub st_mtime: UnsignedLong,
    pub __unused2: UnsignedLong,
    pub st_ctime: UnsignedLong,
    pub __unused3: UnsignedLong,
    pub __unused4: UnsignedLong,
    pub __unused5: UnsignedLong,
}

/// Linux `struct utimbuf` used by the `utime` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Utimbuf {
    pub actime: TimeT,
    pub modtime: TimeT,
}

/// Linux `struct timeval` / `struct __kernel_old_timeval` used by the x86
/// `gettimeofday` and `settimeofday` syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timeval {
    pub tv_sec: TimeT,
    pub tv_usec: Long,
}

/// Linux `ITIMER_REAL`.
pub const ITIMER_REAL: Int = 0;

/// Linux `ITIMER_VIRTUAL`.
pub const ITIMER_VIRTUAL: Int = 1;

/// Linux `ITIMER_PROF`.
pub const ITIMER_PROF: Int = 2;

/// Linux `struct itimerval` used by the historical `getitimer` and
/// `setitimer` syscall ABIs on x86.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Itimerval {
    pub it_interval: Timeval,
    pub it_value: Timeval,
}

/// Linux `fd_set` prefix layout accepted by x86 `select` / `old_select`.
///
/// Linux 1.0 used an 8-word / 256-bit layout, but current x86 kernels define
/// `__FD_SETSIZE` as `1024`, which is 32 `unsigned long` words on 32-bit x86.
/// This struct models that 1024-bit prefix. Current kernels size descriptor
/// set copies from the clipped `nfds` argument, so callers that pass
/// descriptor sets covering more than 1024 fds must provide larger contiguous
/// bitmap storage starting at the same address as this prefix.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FdSet {
    pub fds_bits: [UnsignedLong; 32],
}

/// Linux `struct timezone` used by the historical `gettimeofday` and
/// `settimeofday` syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timezone {
    pub tz_minuteswest: Int,
    pub tz_dsttime: Int,
}

/// Linux `ADJ_OFFSET`.
pub const ADJ_OFFSET: UnsignedInt = 0x0001;

/// Linux `ADJ_FREQUENCY`.
pub const ADJ_FREQUENCY: UnsignedInt = 0x0002;

/// Linux `ADJ_MAXERROR`.
pub const ADJ_MAXERROR: UnsignedInt = 0x0004;

/// Linux `ADJ_ESTERROR`.
pub const ADJ_ESTERROR: UnsignedInt = 0x0008;

/// Linux `ADJ_STATUS`.
pub const ADJ_STATUS: UnsignedInt = 0x0010;

/// Linux `ADJ_TIMECONST`.
pub const ADJ_TIMECONST: UnsignedInt = 0x0020;

/// Linux `ADJ_TAI`.
pub const ADJ_TAI: UnsignedInt = 0x0080;

/// Linux `ADJ_SETOFFSET`.
pub const ADJ_SETOFFSET: UnsignedInt = 0x0100;

/// Linux `ADJ_MICRO`.
pub const ADJ_MICRO: UnsignedInt = 0x1000;

/// Linux `ADJ_NANO`.
pub const ADJ_NANO: UnsignedInt = 0x2000;

/// Linux `ADJ_TICK`.
pub const ADJ_TICK: UnsignedInt = 0x4000;

/// Historical Linux `ADJ_OFFSET_SINGLESHOT`.
pub const ADJ_OFFSET_SINGLESHOT: UnsignedInt = 0x8001;

/// Linux `TIME_OK`.
pub const TIME_OK: Int = 0;

/// Linux `TIME_INS`.
pub const TIME_INS: Int = 1;

/// Linux `TIME_DEL`.
pub const TIME_DEL: Int = 2;

/// Linux `TIME_OOP`.
pub const TIME_OOP: Int = 3;

/// Linux `TIME_WAIT`.
pub const TIME_WAIT: Int = 4;

/// Linux `TIME_ERROR`.
pub const TIME_ERROR: Int = 5;

/// Historical Linux `TIME_BAD`.
pub const TIME_BAD: Int = 4;

/// Linux `struct timex` / `old_timex32` used by the x86 `adjtimex` syscall
/// ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timex {
    pub modes: UnsignedInt,
    pub offset: Long,
    pub freq: Long,
    pub maxerror: Long,
    pub esterror: Long,
    pub status: Int,
    pub constant: Long,
    pub precision: Long,
    pub tolerance: Long,
    pub time: Timeval,
    pub tick: Long,
    pub ppsfreq: Long,
    pub jitter: Long,
    pub shift: Int,
    pub stabil: Long,
    pub jitcnt: Long,
    pub calcnt: Long,
    pub errcnt: Long,
    pub stbcnt: Long,
    pub tai: Int,
    pub __padding: [Int; 11],
}

/// Linux `struct vm86_regs` used by the historical x86 `vm86` / `vm86old`
/// syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vm86Regs {
    pub ebx: Long,
    pub ecx: Long,
    pub edx: Long,
    pub esi: Long,
    pub edi: Long,
    pub ebp: Long,
    pub eax: Long,
    pub __null_ds: Long,
    pub __null_es: Long,
    pub __null_fs: Long,
    pub __null_gs: Long,
    pub orig_eax: Long,
    pub eip: Long,
    pub cs: UnsignedShort,
    pub __csh: UnsignedShort,
    pub eflags: Long,
    pub esp: Long,
    pub ss: UnsignedShort,
    pub __ssh: UnsignedShort,
    pub es: UnsignedShort,
    pub __esh: UnsignedShort,
    pub ds: UnsignedShort,
    pub __dsh: UnsignedShort,
    pub fs: UnsignedShort,
    pub __fsh: UnsignedShort,
    pub gs: UnsignedShort,
    pub __gsh: UnsignedShort,
}

/// Linux `struct revectored_struct` used by the x86 `vm86` / `vm86old`
/// syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RevectoredStruct {
    pub __map: [UnsignedLong; 8],
}

/// Linux `struct vm86_struct` used by the current x86 `vm86old` ABI and still
/// accepted by the original Linux 1.0 `sys_vm86` entry, which consumes only
/// the prefix through `screen_bitmap`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vm86Struct {
    pub regs: Vm86Regs,
    pub flags: UnsignedLong,
    pub screen_bitmap: UnsignedLong,
    pub cpu_type: UnsignedLong,
    pub int_revectored: RevectoredStruct,
    pub int21_revectored: RevectoredStruct,
}

/// Linux `VM86_SCREEN_BITMAP`.
pub const VM86_SCREEN_BITMAP: UnsignedLong = 0x0001;

/// Linux `struct oldold_utsname` used by the historical `oldolduname` syscall
/// ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OldOldUtsname {
    pub sysname: [Char; 9],
    pub nodename: [Char; 9],
    pub release: [Char; 9],
    pub version: [Char; 9],
    pub machine: [Char; 9],
}

/// Linux `struct old_utsname` used by the i386 `olduname` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OldUtsname {
    pub sysname: [Char; 65],
    pub nodename: [Char; 65],
    pub release: [Char; 65],
    pub version: [Char; 65],
    pub machine: [Char; 65],
}

/// Linux `struct kernel_sym` used by the historical `get_kernel_syms`
/// syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KernelSym {
    pub value: UnsignedLong,
    pub name: [Char; 60],
}

/// Linux 1.0 `struct mod_routines` used by the historical `init_module`
/// syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModRoutines {
    /// Kernel-callable module initialization routine address.
    pub init: usize,
    /// Kernel-callable module cleanup routine address.
    pub cleanup: usize,
}

/// Linux `struct new_utsname` used by the i386 `newuname` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NewUtsname {
    pub sysname: [Char; 65],
    pub nodename: [Char; 65],
    pub release: [Char; 65],
    pub version: [Char; 65],
    pub machine: [Char; 65],
    pub domainname: [Char; 65],
}

/// Linux `struct old_sigaction` used by the i386 `sigaction` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OldSigaction {
    /// User-space handler value, including special dispositions such as
    /// `SIG_DFL` (`0`) and `SIG_IGN` (`1`).
    pub sa_handler: usize,
    pub sa_mask: OldSigsetT,
    pub sa_flags: UnsignedLong,
    /// User-space restorer address passed through to the kernel as data.
    pub sa_restorer: usize,
}

/// Linux `struct tms` used by the `times` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tms {
    pub tms_utime: Long,
    pub tms_stime: Long,
    pub tms_cutime: Long,
    pub tms_cstime: Long,
}

pub mod linux_1_0 {
    use super::Int;

    /// Linux 1.0 `struct rlimit` used by the historical `getrlimit` and
    /// `setrlimit` syscall ABIs on x86.
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Rlimit {
        pub rlim_cur: Int,
        pub rlim_max: Int,
    }
}

/// Current Linux i386 `struct rlimit` used by the `setrlimit` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rlimit {
    pub rlim_cur: UnsignedLong,
    pub rlim_max: UnsignedLong,
}

/// Linux `struct rusage`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rusage {
    pub ru_utime: Timeval,
    pub ru_stime: Timeval,
    pub ru_maxrss: Long,
    pub ru_ixrss: Long,
    pub ru_idrss: Long,
    pub ru_isrss: Long,
    pub ru_minflt: Long,
    pub ru_majflt: Long,
    pub ru_nswap: Long,
    pub ru_inblock: Long,
    pub ru_oublock: Long,
    pub ru_msgsnd: Long,
    pub ru_msgrcv: Long,
    pub ru_nsignals: Long,
    pub ru_nvcsw: Long,
    pub ru_nivcsw: Long,
}

/// Linux `struct ustat` used by the historical `ustat` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ustat {
    pub f_tfree: Int,
    pub f_tinode: UnsignedLong,
    pub f_fname: [Char; 6],
    pub f_fpack: [Char; 6],
}

/// Linux 1.0 `struct sysinfo` used by the historical `sysinfo` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sysinfo {
    pub uptime: Long,
    pub loads: [UnsignedLong; 3],
    pub totalram: UnsignedLong,
    pub freeram: UnsignedLong,
    pub sharedram: UnsignedLong,
    pub bufferram: UnsignedLong,
    pub totalswap: UnsignedLong,
    pub freeswap: UnsignedLong,
    pub procs: UnsignedShort,
    pub _f: [Char; 22],
}

/// Linux `fsid_t` used by the historical `statfs` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FsidT {
    pub val: [Long; 2],
}

/// Linux `struct statfs` used by the historical `statfs` and `fstatfs`
/// syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Statfs {
    pub f_type: Long,
    pub f_bsize: Long,
    pub f_blocks: Long,
    pub f_bfree: Long,
    pub f_bavail: Long,
    pub f_files: Long,
    pub f_ffree: Long,
    pub f_fsid: FsidT,
    pub f_namelen: Long,
    pub f_spare: [Long; 6],
}

/// Linux `struct dirent` used by the historical `readdir` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Dirent {
    pub d_ino: Long,
    pub d_off: OffT,
    pub d_reclen: UnsignedShort,
    pub d_name: [Char; 256],
}
