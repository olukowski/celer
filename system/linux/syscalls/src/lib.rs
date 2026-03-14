#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "x86_64")]
mod x86_64;

pub mod arch {
    #[cfg(target_arch = "aarch64")]
    pub use super::aarch64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::x86_64::*;
}

use libc::{
    c_char, c_int, c_long, c_ulong, c_void, mode_t, off_t, pid_t, size_t,
    ssize_t, timex,
};

#[cfg(not(target_arch = "aarch64"))]
use libc::c_uint;

#[cfg(not(miri))]
use arch::{
    Sysno, syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6,
};

#[cfg(miri)]
use core::ptr;

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

/// <https://man7.org/linux/man-pages/man2/acct.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]),
///   or [`core::ptr::null()`] to disable accounting.
pub unsafe fn acct(path: *const c_char) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Acct, path.addr()) } as _;

    #[cfg(miri)]
    {
        _ = path;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/adjtimex.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `buf` must be a valid pointer to a [`timex`] struct,
///   that must be readable/writable until the syscall completes
///   (see [`core::ptr::read`] and [`core::ptr::write`] for details).
pub unsafe fn adjtimex(buf: *mut timex) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Adjtimex, buf.addr()) } as _;

    #[cfg(miri)]
    {
        _ = buf;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/alarm.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
#[cfg(not(target_arch = "aarch64"))]
pub fn alarm(seconds: c_uint) -> c_uint {
    // SAFETY: alarm is safe to call.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Alarm, seconds as _) } as _;

    // Syscall not supported by Miri
    #[cfg(miri)]
    {
        _ = seconds;
        0
    }
}

/// <https://man7.org/linux/man-pages/man2/brk.2.html>
///
/// Returns the raw kernel return value.
///
/// # Safety
/// On success, any pointers or references to memory with an address greater
/// than or equal to `addr` are no longer valid if `addr` is less than the
/// current program break value.
pub unsafe fn brk(addr: *mut c_void) -> *mut c_void {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Brk, addr.addr()) } as _;

    #[cfg(miri)]
    {
        _ = addr;
        // Syscall not supported by Miri
        ptr::null_mut()
    }
}

/// <https://man7.org/linux/man-pages/man2/chdir.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
pub unsafe fn chdir(path: *const c_char) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Chdir, path.addr()) } as _;

    #[cfg(miri)]
    {
        _ = path;
        // Syscall not supported by Miri (when isolation is enabled)
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/chroot.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
pub unsafe fn chroot(path: *const c_char) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall1(Sysno::Chroot, path.addr()) } as _;

    #[cfg(miri)]
    {
        _ = path;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
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

/// <https://man7.org/linux/man-pages/man2/access.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn access(path: *const c_char, mode: c_int) -> c_int {
    // SAFETY: access is safe to call.
    #[cfg(not(miri))]
    return unsafe { syscall2(Sysno::Access, path.addr(), mode as _) } as _;

    #[cfg(miri)]
    {
        _ = path;
        _ = mode;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/chmod.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall2(Sysno::Chmod, path.addr(), mode as _) } as _;

    #[cfg(miri)]
    {
        _ = path;
        _ = mode;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/creat.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `path` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn creat(path: *const c_char, mode: mode_t) -> c_int {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall2(Sysno::Creat, path.addr(), mode as _) } as _;

    #[cfg(miri)]
    {
        _ = path;
        _ = mode;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_int
    }
}

/// <https://man7.org/linux/man-pages/man2/create_module.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - `name` must a pointer to a null-terminated string,
///   that must be readable until the null terminator (see [`core::ptr::read`]).
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn create_module(name: *const c_char, size: size_t) -> c_long {
    // SAFETY: guaranteed by caller.
    #[cfg(not(miri))]
    return unsafe { syscall2(Sysno::CreateModule, name.addr(), size as _) }
        as _;

    #[cfg(miri)]
    {
        _ = name;
        _ = size;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_long
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

/// <https://man7.org/linux/man-pages/man2/clone.2.html>
///
/// Returns the raw kernel return value.
/// Negative values in `[-4095, -1]` represent `errno`.
///
/// # Safety
/// - If this call succeeds, execution continues in both the parent and the
///   child. The caller must ensure that running the child with the provided
///   `flags` is sound.
/// - If the child will run on `stack`, `stack` must point to writable memory
///   that is valid for use as the child stack.
/// - If `flags` contains [`libc::CLONE_PARENT_SETTID`], `parent_tid` must be a
///   valid writable pointer to a [`c_int`] until the syscall completes.
/// - If `flags` contains [`libc::CLONE_CHILD_SETTID`], `child_tid` must point
///   to writable memory where the kernel may store the child TID in the child.
/// - If `flags` contains [`libc::CLONE_CHILD_CLEARTID`], `child_tid` must
///   point to writable memory that remains valid until the child exits,
///   because the kernel may clear it at thread exit.
/// - If `flags` contains [`libc::CLONE_SETTLS`], `tls` must be a valid TLS
///   value for the new thread.
/// - If the child shares memory with the parent, the caller must ensure that no
///   pointers, references, or other Rust aliasing assumptions are violated.
pub unsafe fn clone(
    flags: c_ulong,
    stack: *mut c_void,
    parent_tid: *mut c_int,
    child_tid: *mut c_int,
    tls: c_ulong,
) -> c_long {
    #[cfg(not(miri))]
    {
        #[cfg(target_arch = "x86_64")]
        return unsafe {
            syscall5(
                Sysno::Clone,
                flags as _,
                stack.addr(),
                parent_tid.addr(),
                child_tid.addr(),
                tls as _,
            )
        } as _;

        #[cfg(target_arch = "aarch64")]
        return unsafe {
            syscall5(
                Sysno::Clone,
                flags as _,
                stack.addr(),
                parent_tid.addr(),
                tls as _,
                child_tid.addr(),
            )
        } as _;
    }

    #[cfg(miri)]
    {
        _ = flags;
        _ = stack;
        _ = parent_tid;
        _ = child_tid;
        _ = tls;
        // Syscall not supported by Miri
        -libc::ENOSYS as c_long
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
/// - If `flags` contains [`libc::MAP_FIXED`], the range `[addr, addr + length)` must
///   not overlap any existing mapping that should be preserved; the kernel
///   will silently clobber it, invalidating any pointers or references into
///   that region.
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

#[cfg(test)]
mod tests {
    use core::ptr;

    use super::{close, getpid, mmap, mremap, write};

    #[cfg(not(any(miri, target_arch = "aarch64")))]
    use super::{access, alarm, chmod, creat, create_module};

    #[cfg(not(miri))]
    use {
        super::{acct, adjtimex, brk, chdir, chroot, clone, kill, openat},
        core::mem::MaybeUninit,
        libc::{c_ulong, timex},
    };

    #[test]
    fn test_getpid() {
        assert!(getpid() > 0);
    }

    #[test]
    #[cfg(not(miri))]
    fn test_acct() {
        // will fail due to lack of permission
        let ret = unsafe { acct(ptr::null()) };

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(miri))]
    fn test_adjtimex() {
        let mut tx: MaybeUninit<timex> = MaybeUninit::zeroed();

        // SAFETY: we are passing a valid pointer to a `timex` struct.
        let ret = unsafe { adjtimex((&raw mut tx).cast()) };

        assert!(ret >= 0);
    }

    #[test]
    #[cfg(not(any(miri, target_arch = "aarch64")))]
    fn test_alarm() {
        _ = alarm(0); // cancel any pending alarm
    }

    #[test]
    #[cfg(not(miri))]
    fn test_brk() {
        // SAFETY: we are passing a obviously invalid pointer to `brk`, which
        // should return the current program break instead of changing it.
        _ = unsafe { brk(ptr::null_mut()) };
    }

    #[test]
    #[cfg(not(miri))]
    fn test_chdir() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { chdir(c"".as_ptr()) };

        assert!(ret < 0); // chdir should fail with an empty path
    }

    #[test]
    #[cfg(not(miri))]
    fn test_chroot() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { chroot(c"".as_ptr()) };

        // chroot should fail with an empty path, and lack
        // of permissions
        assert!(ret < 0);
    }

    #[test]
    fn test_close() {
        let ret = close(-1);

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(any(miri, target_arch = "aarch64")))]
    fn test_access() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { access(c"".as_ptr(), 0) };

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(any(miri, target_arch = "aarch64")))]
    fn test_chmod() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { chmod(c"".as_ptr(), 0) };

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(any(miri, target_arch = "aarch64")))]
    fn test_creat() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { creat(c"".as_ptr(), 0) };

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(any(miri, target_arch = "aarch64")))]
    fn test_create_module() {
        // SAFETY: the pointer we are passing is readable until the null
        // terminator (which is the only byte).
        let ret = unsafe { create_module(c"".as_ptr(), 0) };

        // will most likely be -ENOSYS since the syscall was removed in
        // Linux 2.6.
        assert!(ret < 0);
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

        assert!(ret >= 0);
        assert!(ret as usize <= msg.len());
    }

    #[test]
    #[cfg(not(miri))]
    fn test_openat() {
        // SAFETY: the provided path pointer is readable until the null terminator.
        let ret =
            unsafe { openat(libc::AT_FDCWD, c"".as_ptr(), libc::O_RDONLY, 0) };

        assert!(ret < 0);
    }

    #[test]
    #[cfg(not(miri))]
    fn test_clone() {
        // SAFETY: the call will fail due to invalid flags
        let ret = unsafe {
            clone(
                (libc::CLONE_SIGHAND | libc::CLONE_CLEAR_SIGHAND) as c_ulong,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
            )
        };

        assert!(ret < 0);
    }

    #[test]
    fn test_mremap() {
        // SAFETY: we are not using `libc::MREMAP_FIXED`, and this won't
        // succeed so nothing will be invalidated.
        let ret = unsafe { mremap(ptr::null_mut(), 1, 1, 0, ptr::null_mut()) };

        assert!((ret.addr() as isize) < 0);
    }

    #[test]
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

        assert!((ret.addr() as isize) > 0);
    }
}
