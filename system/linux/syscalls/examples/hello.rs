#![no_std]
#![no_main]

use core::panic::PanicInfo;

use celer_system_linux_syscalls::{exit, write};

#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    let msg = b"Hello World!\n";
    // SAFETY: `buf.as_ptr()` is readable for `msg.len()` bytes.
    unsafe { write(1, msg.as_ptr().cast(), msg.len()) };
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
