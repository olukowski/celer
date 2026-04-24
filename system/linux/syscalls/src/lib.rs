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

mod access;
mod acct;
mod adjtimex;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod alarm;
mod errno;
pub mod sys;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use access::{AccessError, access};
pub use acct::{AcctError, acct};
pub use adjtimex::{AdjtimexError, AdjtimexState, adjtimex};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use alarm::alarm;
pub use errno::Errno;
