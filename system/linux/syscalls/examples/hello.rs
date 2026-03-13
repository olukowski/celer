#![no_std]
#![no_main]

use core::{ffi::c_int, panic::PanicInfo};

#[cfg(not(miri))]
use {
    celer_system_linux_syscalls::arch::{Sysno, syscall1, syscall3},
    core::hint::unreachable_unchecked,
};

fn exit(error_code: c_int) -> ! {
    #[cfg(not(miri))]
    {
        // SAFETY: `Sysno::Exit` is safe to call
        unsafe { syscall1(Sysno::Exit, error_code as usize) };

        // SAFETY: `Sysno::Exit` never returns
        unsafe { unreachable_unchecked() };
    }
    #[cfg(miri)]
    {
        // SAFETY: `exit` is safe
        unsafe { libc::exit(error_code) }
    }
}

/// # Safety
/// - `fd` must be a valid file descriptor
unsafe fn write(fd: c_int, bytes: &[u8]) -> isize {
    #[cfg(not(miri))]
    {
        // SAFETY:
        // Caller guarantees `fd` is valid
        unsafe {
            syscall3(
                Sysno::Write,
                fd as usize,
                bytes.as_ptr().addr(),
                bytes.len(),
            )
        }
    }
    #[cfg(miri)]
    {
        // SAFETY: `write` is safe
        unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) }
    }
}

#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    unsafe { write(1, b"Hello World!\n") };
    exit(0)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1)
}

#[cfg(miri)]
#[unsafe(no_mangle)]
fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    _start()
}
