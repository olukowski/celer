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

/// Linux `struct utimbuf` used by the `utime` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Utimbuf {
    pub actime: TimeT,
    pub modtime: TimeT,
}

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

/// Linux `struct tms` used by the `times` syscall ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tms {
    pub tms_utime: Long,
    pub tms_stime: Long,
    pub tms_cutime: Long,
    pub tms_cstime: Long,
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
