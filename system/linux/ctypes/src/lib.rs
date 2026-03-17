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

/// Equivalent to the `pid_t` type in the Linux kernel.
pub type PidT = Int;
