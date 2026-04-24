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
mod chdir;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod chmod;
mod chroot;
mod close;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod creat;
#[cfg(target_arch = "x86")]
mod create_module;
mod delete_module;
mod dup;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod dup2;
mod errno;
mod execve;
mod exit;
mod fchdir;
mod helpers;
pub mod sys;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use access::{AccessError, access};
pub use acct::{AcctError, acct};
pub use adjtimex::{AdjtimexError, adjtimex};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use alarm::alarm;
pub use brk::{BrkError, brk};
pub use chdir::{ChdirError, chdir};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use chmod::{ChmodError, chmod};
pub use chroot::{ChrootError, chroot};
pub use close::{CloseError, close};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use creat::{CreatError, creat};
pub use delete_module::{DeleteModuleError, delete_module};
pub use dup::{DupError, dup};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use dup2::{Dup2Error, dup2};
pub use errno::Errno;
pub use execve::{ExecveError, execve};
pub use exit::exit;
pub use fchdir::{FchdirError, fchdir};

/// Wrapped historical Linux 1.0 syscall ABIs.
#[cfg(target_arch = "x86")]
pub mod linux_1_0 {
    pub use super::create_module::{CreateModuleError, create_module};
}
