#![cfg_attr(not(test), no_std)]

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

#[cfg(all(test, not(miri)))]
mod tests {
    use super::arch::{
        Sysno, syscall0, syscall1, syscall2, syscall3, syscall4, syscall5,
        syscall6,
    };

    #[test]
    fn test_syscall0() {
        // SAFETY: `Sysno::Getpid` is always safe.
        let pid = unsafe { syscall0(Sysno::Getpid) };

        assert!(pid > 0);
    }

    #[test]
    fn test_syscall1() {
        // SAFETY: `Sysno::Close` with `-1` is always safe.
        let ret = unsafe { syscall1(Sysno::Close, usize::MAX) };

        assert!(ret < 0);
    }

    #[test]
    fn test_syscall2() {
        // SAFETY: `Sysno::Getpid` is always safe.
        let pid = unsafe { syscall0(Sysno::Getpid) };

        // SAFETY: `Sysno::Kill` with `pid` and `0` is always safe.
        let ret = unsafe { syscall2(Sysno::Kill, pid as usize, 0) };

        assert_eq!(ret, 0);
    }

    #[test]
    fn test_syscall3() {
        let msg = b"test\n";

        // SAFETY: `Sysno::Write` with `1` (stdout), `msg.as_ptr().addr()`,
        // and `msg.len()` is safe.
        let ret = unsafe {
            syscall3(Sysno::Write, 1, msg.as_ptr().addr(), msg.len())
        };

        assert_eq!(ret as usize, msg.len());
    }

    #[test]
    fn test_syscall4() {
        // SAFETY: `Sysno::Openat` with `AT_FDCWD`, `""`, `O_RDONLY`,
        // and `0` is safe, but will fail due to empty path.
        let ret = unsafe {
            syscall4(
                Sysno::Openat,
                libc::AT_FDCWD as usize,
                c"".as_ptr().addr(),
                libc::O_RDONLY as usize,
                0,
            )
        };

        assert!(ret < 0);
    }

    #[test]
    fn test_syscall5() {
        // SAFETY: `Sysno::Mremap` with `1`, `1`, `1`, `0`, and `0` is safe.
        let ret = unsafe { syscall5(Sysno::Mremap, 1, 1, 1, 0, 0) };

        assert!(ret < 0);
    }

    #[test]
    fn test_syscall6() {
        // SAFETY: `Sysno::Mmap` with `0`, `4096`, `libc::PROT_READ`,
        // `libc::MAP_PRIVATE | libc::MAP_ANONYMOUS`, `usize::MAX`,
        // and `0` is safe.
        let addr = unsafe {
            syscall6(
                Sysno::Mmap,
                0,
                4096,
                libc::PROT_READ as usize,
                (libc::MAP_PRIVATE | libc::MAP_ANONYMOUS) as usize,
                usize::MAX,
                0,
            )
        };

        assert!(addr > 0);
    }
}
