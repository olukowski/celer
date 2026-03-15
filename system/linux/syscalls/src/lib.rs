#![cfg(target_os = "linux")]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(target_arch = "x86")]
mod x86;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "x86_64")]
mod x86_64;

pub mod arch {
    #[cfg(target_arch = "x86")]
    pub use super::x86::*;

    #[cfg(target_arch = "aarch64")]
    pub use super::aarch64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::x86_64::*;
}

use libc::{c_char, c_int, c_void, mode_t, off_t, pid_t, size_t, ssize_t};

#[cfg(not(miri))]
use arch::{
    Sysno, syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6,
};

#[cfg(all(target_arch = "x86", not(miri)))]
use libc::c_ulong;

/// <https://man7.org/linux/man-pages/man2/getpid.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
pub fn getpid() -> pid_t {
    // SAFETY: getpid is safe to call.
    #[cfg(not(miri))]
    return unsafe { syscall0(Sysno::Getpid) } as _;

    // SAFETY: getpid is safe to call.
    #[cfg(miri)]
    unsafe {
        libc::getpid()
    }
}

/// <https://man7.org/linux/man-pages/man2/close.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
pub fn close(fd: c_int) -> c_int {
    // SAFETY: close is safe to call.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Close, fd as _) } as _;

    // SAFETY: close is safe to call.
    #[cfg(miri)]
    unsafe {
        libc::close(fd)
    }
}

/// <https://man7.org/linux/man-pages/man2/exit.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn exit(status: c_int) -> ! {
    #[cfg(not(miri))]
    {
        use core::hint::unreachable_unchecked;

        // SAFETY: exit is safe to call.
        unsafe { syscall1(Sysno::Exit, status as _) };
        // SAFETY: exit never returns.
        unsafe { unreachable_unchecked() }
    }

    // SAFETY: exit is safe to call.
    #[cfg(miri)]
    unsafe {
        libc::exit(status)
    }
}

/// <https://man7.org/linux/man-pages/man2/kill.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
pub fn kill(pid: pid_t, sig: c_int) -> c_int {
    // SAFETY: kill is safe to call.
    #[cfg(not(miri))]
    return unsafe { syscall2(Sysno::Kill, pid as _, sig as _) } as _;

    #[cfg(miri)]
    {
        _ = pid;
        _ = sig;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/write.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `buf` must be readable for at least `count` bytes (see [`core::ptr::read`]).
pub unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall3(Sysno::Write, fd as _, buf.addr(), count as _) }
        as _;

    // SAFETY: guaranteed by caller.
    #[cfg(miri)]
    unsafe {
        libc::write(fd, buf, count)
    }
}

/// <https://man7.org/linux/man-pages/man2/openat.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
pub unsafe fn openat(
    dirfd: c_int,
    path: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe {
        syscall4(
            Sysno::Openat,
            dirfd as _,
            path.addr(),
            flags as _,
            mode as _,
        )
    } as _;

    #[cfg(miri)]
    {
        _ = dirfd;
        _ = path;
        _ = flags;
        _ = mode;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/mremap.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - If `flags` contains [`libc::MREMAP_FIXED`], the range
///   `[new_address, new_address + new_size)` must not overlap any existing
///   mapping that should be preserved; the kernel will silently clobber it,
///   invalidating any pointers or references into that region.
/// - After a successful call where the mapping was moved, any pointers or
///   references into the old range `[old_address, old_address + old_size)`
///   are invalidated and must not be used.
pub unsafe fn mremap(
    old_address: *mut c_void,
    old_size: size_t,
    new_size: size_t,
    flags: c_int,
    new_address: *mut c_void,
) -> *mut c_void {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe {
        syscall5(
            Sysno::Mremap,
            old_address.addr(),
            old_size as _,
            new_size as _,
            flags as _,
            new_address.addr(),
        )
    } as _;

    // SAFETY: guaranteed by caller.
    #[cfg(miri)]
    unsafe {
        libc::mremap(old_address, old_size, new_size, flags, new_address)
    }
}

/// <https://man7.org/linux/man-pages/man2/mmap.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - If `flags` contains [`libc::MAP_FIXED`], the range `[addr, addr + length)`
///   must not overlap any existing mapping that should be preserved; the kernel
///   will silently clobber it, invalidating any pointers or references into
///   that region.
#[cfg(any(not(target_arch = "x86"), miri))]
pub unsafe fn mmap(
    addr: *mut c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe {
        syscall6(
            Sysno::Mmap,
            addr.addr(),
            length as _,
            prot as _,
            flags as _,
            fd as _,
            offset as _,
        )
    } as _;

    // SAFETY: guaranteed by caller.
    #[cfg(miri)]
    unsafe {
        libc::mmap(addr, length, prot, flags, fd, offset)
    }
}

/// Argument struct for [`old_mmap`].
#[allow(non_camel_case_types)]
#[repr(C)]
#[cfg(all(target_arch = "x86", not(miri)))]
pub struct mmap_arg_struct {
    addr: *mut c_void,
    length: c_ulong,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: c_ulong,
}

/// <https://man7.org/linux/man-pages/man2/mmap.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `args` must be a valid pointer to a [`mmap_arg_struct`],
///   that must be readable until the syscall completes
///   (see [`core::ptr::read`] for details).
/// - If `args.flags` contains [`libc::MAP_FIXED`], the range
///   `[args.addr, args.addr + args.length)`  must not overlap any existing
///   mapping that should be preserved; the kernel will silently clobber it,
///   invalidating any pointers or references into that region.
#[cfg(all(target_arch = "x86", not(miri)))]
pub unsafe fn old_mmap(args: *const mmap_arg_struct) -> *mut c_void {
    // SAFETY: guaranteed by caller.
    (unsafe { syscall1(Sysno::Mmap, args.addr()) }) as _
}

///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - If `flags` contains [`libc::MAP_FIXED`], the range `[addr, addr + length)`
///   must not overlap any existing mapping that should be preserved; the kernel
///   will silently clobber it, invalidating any pointers or references into
///   that region.
#[cfg(all(target_arch = "x86", not(miri)))]
pub unsafe fn mmap2(
    addr: *mut c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall6(
            Sysno::Mmap,
            addr.addr(),
            length as _,
            prot as _,
            flags as _,
            fd as _,
            offset as _,
        )
    }) as _
}

#[cfg(test)]
mod tests {
    use core::ptr;

    use super::{close, getpid, mremap, write};

    #[cfg(any(not(target_arch = "x86"), miri))]
    use super::mmap;

    #[cfg(not(miri))]
    use super::{kill, openat};

    #[cfg(all(target_arch = "x86", not(miri)))]
    use super::{mmap_arg_struct, mmap2, old_mmap};

    fn is_error(ret: isize) -> bool {
        (-4095..0).contains(&ret)
    }

    #[test]
    fn test_getpid() {
        assert!(getpid() > 0);
    }

    #[test]
    fn test_close() {
        let ret = close(-1);

        assert!(is_error(ret as isize));
    }

    #[test]
    #[cfg(not(miri))]
    fn test_kill() {
        let pid = getpid();

        let ret = kill(pid, 0);

        assert_eq!(ret, 0);
    }

    #[test]
    fn test_write() {
        let msg = b"test\n";

        // SAFETY: `buf.as_ptr()` is readable for `msg.len()` bytes.
        let ret = unsafe { write(1, msg.as_ptr().cast(), msg.len()) };

        assert!(!is_error(ret));
        assert!(ret as usize <= msg.len());
    }

    #[test]
    #[cfg(not(miri))]
    fn test_openat() {
        // SAFETY: the provided path pointer is readable until the null terminator.
        let ret =
            unsafe { openat(libc::AT_FDCWD, c"".as_ptr(), libc::O_RDONLY, 0) };

        assert!(is_error(ret as isize));
    }

    #[test]
    fn test_mremap() {
        // SAFETY: we are not using `libc::MREMAP_FIXED`, and this won't
        // succeed so nothing will be invalidated.
        let ret = unsafe { mremap(ptr::null_mut(), 1, 1, 0, ptr::null_mut()) };

        assert!(is_error(ret as isize));
    }

    #[test]
    #[cfg(any(not(target_arch = "x86"), miri))]
    fn test_mmap() {
        // SAFETY: we are not using `libc::MAP_FIXED`.
        let ret = unsafe {
            mmap(
                ptr::null_mut(),
                4096,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        assert!(!is_error(ret as isize));
    }

    #[test]
    #[cfg(all(target_arch = "x86", not(miri)))]
    fn test_old_mmap() {
        let args = mmap_arg_struct {
            addr: ptr::null_mut(),
            length: 4096,
            prot: libc::PROT_READ | libc::PROT_WRITE,
            flags: libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            fd: -1,
            offset: 0,
        };

        // SAFETY: we are not using `libc::MAP_FIXED`, and `args` is a valid
        // pointer to a `mmap_arg_struct`.
        let ret = unsafe { old_mmap(&raw const args) };

        assert!(!is_error(ret as isize));
    }

    #[test]
    #[cfg(all(target_arch = "x86", not(miri)))]
    fn test_mmap2() {
        // SAFETY: we are not using `libc::MAP_FIXED`.
        let ret = unsafe {
            mmap2(
                ptr::null_mut(),
                4096,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        assert!(!is_error(ret as isize));
    }
}
