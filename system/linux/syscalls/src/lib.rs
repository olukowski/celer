#![cfg(not(miri))] // Miri does not support syscalls
#![cfg(target_os = "linux")]
#![cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64"
))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod arch;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod access;
mod acct;
mod adjtimex;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod alarm;
mod brk;
mod errno;
mod helpers;
pub mod sys;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use access::{AccessError, access};
pub use acct::{AcctError, acct};
pub use adjtimex::{AdjtimexError, adjtimex};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use alarm::alarm;
pub use brk::{BrkError, brk};
pub use errno::Errno;
