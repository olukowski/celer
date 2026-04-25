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
mod fchmod;
#[cfg(target_arch = "x86")]
mod fchown;
mod fcntl;
#[cfg(target_arch = "x86")]
mod fork;
mod fstatfs;
mod fsync;
mod ftruncate;
#[cfg(target_arch = "x86")]
mod get_kernel_syms;
#[cfg(target_arch = "x86")]
mod getegid;
#[cfg(target_arch = "x86")]
mod geteuid;
#[cfg(target_arch = "x86")]
mod getgid;
#[cfg(target_arch = "x86")]
mod getgroups;
mod getitimer;
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
pub use fchmod::{FchmodError, fchmod};
#[cfg(target_arch = "x86")]
pub use fchown::{Fchown16Error, fchown16};
pub use fcntl::{FcntlError, fcntl};
#[cfg(target_arch = "x86")]
pub use fork::{ForkError, fork};
pub use fstatfs::{FstatfsError, fstatfs};
pub use fsync::{FsyncError, fsync};
pub use ftruncate::{FtruncateError, ftruncate};
#[cfg(target_arch = "x86")]
pub use get_kernel_syms::{GetKernelSymsError, get_kernel_syms};
#[cfg(target_arch = "x86")]
pub use getegid::getegid16;
#[cfg(target_arch = "x86")]
pub use geteuid::geteuid16;
#[cfg(target_arch = "x86")]
pub use getgid::getgid16;
#[cfg(target_arch = "x86")]
pub use getgroups::{Getgroups16Error, getgroups16};
pub use getitimer::{GetitimerError, getitimer};

/// Wrapped historical Linux 1.0 syscall ABIs.
#[cfg(target_arch = "x86")]
pub mod linux_1_0 {
    pub use super::create_module::{CreateModuleError, create_module};
    pub use super::fstatfs::{FstatfsError, fstatfs_1_0 as fstatfs};
    pub use super::ftruncate::{
        Ftruncate1_0Error as FtruncateError, ftruncate_1_0 as ftruncate,
    };
}
