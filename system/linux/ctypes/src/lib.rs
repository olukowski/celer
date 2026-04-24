#![no_std]
#![cfg(target_os = "linux")]
#![cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

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
#[cfg(target_arch = "x86")]
pub type OldUidT = UnsignedShort;

/// Equivalent to the native `uid_t` type on aarch64.
#[cfg(target_arch = "aarch64")]
pub type OldUidT = UidT;

/// Equivalent to the `gid_t` type in the Linux kernel.
pub type GidT = UnsignedInt;

/// Equivalent to the legacy 16-bit `old_gid_t` type used by the i386
/// `setgid` syscall ABI.
#[cfg(target_arch = "x86")]
pub type OldGidT = UnsignedShort;

/// Equivalent to the native `gid_t` type on aarch64.
#[cfg(target_arch = "aarch64")]
pub type OldGidT = GidT;

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

/// Current i386 Linux `struct stat` used by the `newstat`, `newlstat`, and
/// `newfstat` syscall ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NewStat {
    pub st_dev: UnsignedLong,
    pub st_ino: UnsignedLong,
    pub st_mode: UnsignedShort,
    pub st_nlink: UnsignedShort,
    pub st_uid: UnsignedShort,
    pub st_gid: UnsignedShort,
    pub st_rdev: UnsignedLong,
    pub st_size: UnsignedLong,
    pub st_blksize: UnsignedLong,
    pub st_blocks: UnsignedLong,
    pub st_atime: UnsignedLong,
    pub st_atime_nsec: UnsignedLong,
    pub st_mtime: UnsignedLong,
    pub st_mtime_nsec: UnsignedLong,
    pub st_ctime: UnsignedLong,
    pub st_ctime_nsec: UnsignedLong,
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
    #[cfg(target_arch = "x86")]
    pub fds_bits: [UnsignedLong; 32],
    #[cfg(target_arch = "aarch64")]
    pub fds_bits: [UnsignedLong; 16],
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

#[cfg(target_arch = "x86")]
pub mod linux_1_0 {
    use super::{Char, Int, Long, OffT, UnsignedLong, UnsignedShort};

    /// Linux 1.0 `struct rlimit` used by the historical `getrlimit` and
    /// `setrlimit` syscall ABIs on x86.
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Rlimit {
        pub rlim_cur: Int,
        pub rlim_max: Int,
    }

    /// Linux 1.0 `struct new_stat`.
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

    /// Linux 1.0 `fsid_t`.
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct FsidT {
        pub val: [Long; 2],
    }

    /// Linux 1.0 `struct statfs`.
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

    /// Linux 1.0 `struct sysinfo`.
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

    /// Linux 1.0 `struct dirent` used by the historical `readdir` syscall ABI.
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Dirent {
        pub d_ino: Long,
        pub d_off: OffT,
        pub d_reclen: UnsignedShort,
        pub d_name: [Char; 256],
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

/// Current i386 Linux `struct sysinfo` used by the `sysinfo` syscall ABI.
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
    pub pad: UnsignedShort,
    pub totalhigh: UnsignedLong,
    pub freehigh: UnsignedLong,
    pub mem_unit: UnsignedInt,
    #[cfg(target_arch = "x86")]
    pub _f: [Char; 8],
    #[cfg(target_arch = "aarch64")]
    pub _f: [Char; 0],
}

/// Linux `__kernel_fsid_t` used by current i386 filesystem status ABIs.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FsidT {
    pub val: [Int; 2],
}

/// Current i386 Linux `struct statfs` used by the `statfs` and `fstatfs`
/// syscall ABIs.
#[cfg(target_arch = "x86")]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Statfs {
    pub f_type: UnsignedInt,
    pub f_bsize: UnsignedInt,
    pub f_blocks: UnsignedInt,
    pub f_bfree: UnsignedInt,
    pub f_bavail: UnsignedInt,
    pub f_files: UnsignedInt,
    pub f_ffree: UnsignedInt,
    pub f_fsid: FsidT,
    pub f_namelen: UnsignedInt,
    pub f_frsize: UnsignedInt,
    pub f_flags: UnsignedInt,
    pub f_spare: [UnsignedInt; 4],
}

/// Current aarch64 Linux `struct statfs` used by the `statfs` and `fstatfs`
/// syscall ABIs.
#[cfg(target_arch = "aarch64")]
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
    pub f_frsize: Long,
    pub f_flags: Long,
    pub f_spare: [Long; 4],
}

/// Maximum current-kernel filename payload for the legacy `old_readdir` ABI.
///
/// Current kernels reject names with `namlen >= PATH_MAX`; `PATH_MAX` is
/// `4096`, so this storage is large enough for any successful name plus the
/// trailing NUL byte.
pub const OLD_LINUX_DIRENT_NAME_CAP: usize = 4096;

/// Current i386 Linux `struct old_linux_dirent` buffer used by the historical
/// `old_readdir` syscall ABI.
///
/// The C ABI has a flexible `d_name[]` member at byte offset `10`. This Rust
/// type gives that payload fixed backing storage large enough for any
/// successful current-kernel result.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OldLinuxDirent {
    pub d_ino: Long,
    pub d_offset: OffT,
    pub d_namlen: UnsignedShort,
    pub d_name: [Char; OLD_LINUX_DIRENT_NAME_CAP],
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::{align_of, offset_of, size_of};

    #[cfg(target_arch = "x86")]
    use super::linux_1_0;
    use super::{FsidT, NewStat, OldLinuxDirent, Statfs, Sysinfo};

    #[test]
    fn current_newstat_layout_matches_linux_v7_0_struct_stat() {
        assert_eq!(offset_of!(NewStat, st_dev), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(size_of::<NewStat>(), 64);
            assert_eq!(align_of::<NewStat>(), 4);
            assert_eq!(offset_of!(NewStat, st_ino), 4);
            assert_eq!(offset_of!(NewStat, st_mode), 8);
            assert_eq!(offset_of!(NewStat, st_nlink), 10);
            assert_eq!(offset_of!(NewStat, st_uid), 12);
            assert_eq!(offset_of!(NewStat, st_gid), 14);
            assert_eq!(offset_of!(NewStat, st_rdev), 16);
            assert_eq!(offset_of!(NewStat, st_size), 20);
            assert_eq!(offset_of!(NewStat, st_blksize), 24);
            assert_eq!(offset_of!(NewStat, st_blocks), 28);
            assert_eq!(offset_of!(NewStat, st_atime), 32);
            assert_eq!(offset_of!(NewStat, st_atime_nsec), 36);
            assert_eq!(offset_of!(NewStat, st_mtime), 40);
            assert_eq!(offset_of!(NewStat, st_mtime_nsec), 44);
            assert_eq!(offset_of!(NewStat, st_ctime), 48);
            assert_eq!(offset_of!(NewStat, st_ctime_nsec), 52);
            assert_eq!(offset_of!(NewStat, __unused4), 56);
            assert_eq!(offset_of!(NewStat, __unused5), 60);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert_eq!(size_of::<NewStat>(), 120);
            assert_eq!(align_of::<NewStat>(), 8);
            assert_eq!(offset_of!(NewStat, st_ino), 8);
            assert_eq!(offset_of!(NewStat, st_mode), 16);
            assert_eq!(offset_of!(NewStat, st_nlink), 18);
            assert_eq!(offset_of!(NewStat, st_uid), 20);
            assert_eq!(offset_of!(NewStat, st_gid), 22);
            assert_eq!(offset_of!(NewStat, st_rdev), 24);
            assert_eq!(offset_of!(NewStat, st_size), 32);
            assert_eq!(offset_of!(NewStat, st_blksize), 40);
            assert_eq!(offset_of!(NewStat, st_blocks), 48);
            assert_eq!(offset_of!(NewStat, st_atime), 56);
            assert_eq!(offset_of!(NewStat, st_atime_nsec), 64);
            assert_eq!(offset_of!(NewStat, st_mtime), 72);
            assert_eq!(offset_of!(NewStat, st_mtime_nsec), 80);
            assert_eq!(offset_of!(NewStat, st_ctime), 88);
            assert_eq!(offset_of!(NewStat, st_ctime_nsec), 96);
            assert_eq!(offset_of!(NewStat, __unused4), 104);
            assert_eq!(offset_of!(NewStat, __unused5), 112);
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn linux_1_0_newstat_layout_is_preserved() {
        assert_eq!(size_of::<linux_1_0::NewStat>(), 64);
        assert_eq!(align_of::<linux_1_0::NewStat>(), 4);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_dev), 0);
        assert_eq!(offset_of!(linux_1_0::NewStat, __pad1), 2);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_ino), 4);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_mode), 8);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_nlink), 10);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_uid), 12);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_gid), 14);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_rdev), 16);
        assert_eq!(offset_of!(linux_1_0::NewStat, __pad2), 18);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_size), 20);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_blksize), 24);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_blocks), 28);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_atime), 32);
        assert_eq!(offset_of!(linux_1_0::NewStat, __unused1), 36);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_mtime), 40);
        assert_eq!(offset_of!(linux_1_0::NewStat, __unused2), 44);
        assert_eq!(offset_of!(linux_1_0::NewStat, st_ctime), 48);
        assert_eq!(offset_of!(linux_1_0::NewStat, __unused3), 52);
        assert_eq!(offset_of!(linux_1_0::NewStat, __unused4), 56);
        assert_eq!(offset_of!(linux_1_0::NewStat, __unused5), 60);
    }

    #[test]
    fn current_statfs_layout_matches_linux_v7_0_struct_statfs() {
        assert_eq!(size_of::<FsidT>(), 8);
        assert_eq!(align_of::<FsidT>(), 4);
        assert_eq!(offset_of!(Statfs, f_type), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(size_of::<Statfs>(), 64);
            assert_eq!(align_of::<Statfs>(), 4);
            assert_eq!(offset_of!(Statfs, f_bsize), 4);
            assert_eq!(offset_of!(Statfs, f_blocks), 8);
            assert_eq!(offset_of!(Statfs, f_bfree), 12);
            assert_eq!(offset_of!(Statfs, f_bavail), 16);
            assert_eq!(offset_of!(Statfs, f_files), 20);
            assert_eq!(offset_of!(Statfs, f_ffree), 24);
            assert_eq!(offset_of!(Statfs, f_fsid), 28);
            assert_eq!(offset_of!(Statfs, f_namelen), 36);
            assert_eq!(offset_of!(Statfs, f_frsize), 40);
            assert_eq!(offset_of!(Statfs, f_flags), 44);
            assert_eq!(offset_of!(Statfs, f_spare), 48);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert_eq!(size_of::<Statfs>(), 120);
            assert_eq!(align_of::<Statfs>(), 8);
            assert_eq!(offset_of!(Statfs, f_bsize), 8);
            assert_eq!(offset_of!(Statfs, f_blocks), 16);
            assert_eq!(offset_of!(Statfs, f_bfree), 24);
            assert_eq!(offset_of!(Statfs, f_bavail), 32);
            assert_eq!(offset_of!(Statfs, f_files), 40);
            assert_eq!(offset_of!(Statfs, f_ffree), 48);
            assert_eq!(offset_of!(Statfs, f_fsid), 56);
            assert_eq!(offset_of!(Statfs, f_namelen), 64);
            assert_eq!(offset_of!(Statfs, f_frsize), 72);
            assert_eq!(offset_of!(Statfs, f_flags), 80);
            assert_eq!(offset_of!(Statfs, f_spare), 88);
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn linux_1_0_statfs_layout_is_preserved() {
        assert_eq!(size_of::<linux_1_0::FsidT>(), 8);
        assert_eq!(align_of::<linux_1_0::FsidT>(), 4);
        assert_eq!(size_of::<linux_1_0::Statfs>(), 64);
        assert_eq!(align_of::<linux_1_0::Statfs>(), 4);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_type), 0);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_bsize), 4);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_blocks), 8);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_bfree), 12);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_bavail), 16);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_files), 20);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_ffree), 24);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_fsid), 28);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_namelen), 36);
        assert_eq!(offset_of!(linux_1_0::Statfs, f_spare), 40);
    }

    #[test]
    fn current_sysinfo_layout_matches_linux_v7_0_struct_sysinfo() {
        assert_eq!(offset_of!(Sysinfo, uptime), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(size_of::<Sysinfo>(), 64);
            assert_eq!(align_of::<Sysinfo>(), 4);
            assert_eq!(offset_of!(Sysinfo, loads), 4);
            assert_eq!(offset_of!(Sysinfo, totalram), 16);
            assert_eq!(offset_of!(Sysinfo, freeram), 20);
            assert_eq!(offset_of!(Sysinfo, sharedram), 24);
            assert_eq!(offset_of!(Sysinfo, bufferram), 28);
            assert_eq!(offset_of!(Sysinfo, totalswap), 32);
            assert_eq!(offset_of!(Sysinfo, freeswap), 36);
            assert_eq!(offset_of!(Sysinfo, procs), 40);
            assert_eq!(offset_of!(Sysinfo, pad), 42);
            assert_eq!(offset_of!(Sysinfo, totalhigh), 44);
            assert_eq!(offset_of!(Sysinfo, freehigh), 48);
            assert_eq!(offset_of!(Sysinfo, mem_unit), 52);
            assert_eq!(offset_of!(Sysinfo, _f), 56);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert_eq!(size_of::<Sysinfo>(), 112);
            assert_eq!(align_of::<Sysinfo>(), 8);
            assert_eq!(offset_of!(Sysinfo, loads), 8);
            assert_eq!(offset_of!(Sysinfo, totalram), 32);
            assert_eq!(offset_of!(Sysinfo, freeram), 40);
            assert_eq!(offset_of!(Sysinfo, sharedram), 48);
            assert_eq!(offset_of!(Sysinfo, bufferram), 56);
            assert_eq!(offset_of!(Sysinfo, totalswap), 64);
            assert_eq!(offset_of!(Sysinfo, freeswap), 72);
            assert_eq!(offset_of!(Sysinfo, procs), 80);
            assert_eq!(offset_of!(Sysinfo, pad), 82);
            assert_eq!(offset_of!(Sysinfo, totalhigh), 88);
            assert_eq!(offset_of!(Sysinfo, freehigh), 96);
            assert_eq!(offset_of!(Sysinfo, mem_unit), 104);
            assert_eq!(offset_of!(Sysinfo, _f), 108);
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn linux_1_0_sysinfo_layout_is_preserved() {
        assert_eq!(size_of::<linux_1_0::Sysinfo>(), 64);
        assert_eq!(align_of::<linux_1_0::Sysinfo>(), 4);
        assert_eq!(offset_of!(linux_1_0::Sysinfo, uptime), 0);
        assert_eq!(offset_of!(linux_1_0::Sysinfo, loads), 4);
        assert_eq!(offset_of!(linux_1_0::Sysinfo, totalram), 16);
        assert_eq!(offset_of!(linux_1_0::Sysinfo, procs), 40);
        assert_eq!(offset_of!(linux_1_0::Sysinfo, _f), 42);
    }

    #[test]
    fn current_old_linux_dirent_prefix_layout_matches_linux_v7_0() {
        assert_eq!(offset_of!(OldLinuxDirent, d_ino), 0);
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(size_of::<OldLinuxDirent>(), 4108);
            assert_eq!(align_of::<OldLinuxDirent>(), 4);
            assert_eq!(offset_of!(OldLinuxDirent, d_offset), 4);
            assert_eq!(offset_of!(OldLinuxDirent, d_namlen), 8);
            assert_eq!(offset_of!(OldLinuxDirent, d_name), 10);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert_eq!(size_of::<OldLinuxDirent>(), 4120);
            assert_eq!(align_of::<OldLinuxDirent>(), 8);
            assert_eq!(offset_of!(OldLinuxDirent, d_offset), 8);
            assert_eq!(offset_of!(OldLinuxDirent, d_namlen), 16);
            assert_eq!(offset_of!(OldLinuxDirent, d_name), 18);
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    fn linux_1_0_dirent_layout_is_preserved() {
        assert_eq!(size_of::<linux_1_0::Dirent>(), 268);
        assert_eq!(align_of::<linux_1_0::Dirent>(), 4);
        assert_eq!(offset_of!(linux_1_0::Dirent, d_ino), 0);
        assert_eq!(offset_of!(linux_1_0::Dirent, d_off), 4);
        assert_eq!(offset_of!(linux_1_0::Dirent, d_reclen), 8);
        assert_eq!(offset_of!(linux_1_0::Dirent, d_name), 10);
    }
}
