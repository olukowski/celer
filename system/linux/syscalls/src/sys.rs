//! Linux system calls.
//!
//! Each system call documents its own:
//! - Behavior
//! - Kernel support
//! - Required privileges
//! - Errors
//!
//! Also, each system call has references to the "latest" source code of the
//! system call in the Linux kernel. The exact definition of "latest" here
//! is: the latest version at the time of writing.
//!
//! The return value of a system call is the raw kernel return value.
//! Negative values in the range `[-4095, -1]` indicate errno codes;
//! the caller is responsible for interpreting them, except for syscalls such
//! as `brk` that return raw address values instead of signed errno-shaped
//! values.
//! Note: there are some system calls that cannot fail.

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(target_arch = "x86")]
mod access;
mod acct;
mod adjtimex;
#[cfg(target_arch = "x86")]
mod alarm;
mod brk;
mod chdir;
#[cfg(target_arch = "x86")]
mod chmod;
mod chroot;
mod close;
#[cfg(target_arch = "x86")]
mod creat;
#[cfg(target_arch = "x86")]
mod create_module;
mod delete_module;
mod dup;
#[cfg(target_arch = "x86")]
mod dup2;
mod execve;
mod exit;
mod fchdir;
mod fchmod;
#[cfg(target_arch = "x86")]
mod fchown;
mod fcntl;
#[cfg(target_arch = "x86")]
mod fork;
#[cfg(target_arch = "x86")]
mod fstat;
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
mod getpgid;
#[cfg(target_arch = "x86")]
mod getpgrp;
mod getpid;
mod getppid;
mod getpriority;
mod getrlimit;
mod getrusage;
mod gettimeofday;
#[cfg(target_arch = "x86")]
mod getuid;
#[cfg(target_arch = "x86")]
mod idle;
mod init_module;
mod ioctl;
#[cfg(target_arch = "x86")]
mod ioperm;
#[cfg(target_arch = "x86")]
mod iopl;
#[cfg(target_arch = "x86")]
mod ipc;
mod kill;
#[cfg(target_arch = "x86")]
mod lchown;
#[cfg(target_arch = "x86")]
mod link;
mod lseek;
#[cfg(target_arch = "x86")]
mod lstat;
#[cfg(target_arch = "x86")]
mod mkdir;
#[cfg(target_arch = "x86")]
mod mknod;
mod mmap;
#[cfg(target_arch = "x86")]
mod modify_ldt;
mod mount;
mod munmap;
mod newfstat;
#[cfg(target_arch = "x86")]
mod newlstat;
#[cfg(target_arch = "x86")]
mod newstat;
#[cfg(target_arch = "x86")]
mod nice;
#[cfg(target_arch = "x86")]
mod open;
#[cfg(target_arch = "x86")]
mod pause;
#[cfg(target_arch = "x86")]
mod pipe;
mod ptrace;
mod read;
#[cfg(target_arch = "x86")]
mod readdir;
#[cfg(target_arch = "x86")]
mod readlink;
mod reboot;
#[cfg(target_arch = "x86")]
mod rename;
#[cfg(target_arch = "x86")]
mod rmdir;
#[cfg(target_arch = "x86")]
mod select;
mod setdomainname;
#[cfg(target_arch = "x86")]
mod setgid;
#[cfg(target_arch = "x86")]
mod setgroups;
mod sethostname;
mod setitimer;
mod setpgid;
mod setpriority;
#[cfg(target_arch = "x86")]
mod setregid;
#[cfg(target_arch = "x86")]
mod setreuid;
mod setrlimit;
mod setsid;
mod settimeofday;
#[cfg(target_arch = "x86")]
mod setuid;
#[cfg(target_arch = "x86")]
mod setup;
#[cfg(target_arch = "x86")]
mod sgetmask;
#[cfg(target_arch = "x86")]
mod sigaction;
#[cfg(target_arch = "x86")]
mod signal;
#[cfg(target_arch = "x86")]
mod sigpending;
#[cfg(target_arch = "x86")]
mod sigprocmask;
#[cfg(target_arch = "x86")]
mod sigreturn;
#[cfg(target_arch = "x86")]
mod sigsuspend;
#[cfg(target_arch = "x86")]
mod socketcall;
#[cfg(target_arch = "x86")]
mod ssetmask;
#[cfg(target_arch = "x86")]
mod stat;
mod statfs;
#[cfg(target_arch = "x86")]
mod stime;
mod swapoff;
mod swapon;
#[cfg(target_arch = "x86")]
mod symlink;
mod sync;
mod sysinfo;
mod syslog;
#[cfg(target_arch = "x86")]
mod time;
mod times;
mod truncate;
mod umask;
mod umount;
mod uname;
#[cfg(target_arch = "x86")]
mod unlink;
#[cfg(target_arch = "x86")]
mod uselib;
#[cfg(target_arch = "x86")]
mod ustat;
#[cfg(target_arch = "x86")]
mod utime;
mod vhangup;
#[cfg(target_arch = "x86")]
mod vm86;
mod wait4;
#[cfg(target_arch = "x86")]
mod waitpid;
mod write;

/// Historical syscall wrappers that are only correct for Linux 1.0 ABIs.
///
/// These wrappers are intentionally kept out of the default `crate::sys` namespace
/// when the same numeric slot gained different semantics on newer kernels.
#[cfg(target_arch = "x86")]
pub mod linux_1_0 {
    pub use super::fstatfs::fstatfs_1_0 as fstatfs;
    pub use super::ftruncate::ftruncate_1_0 as ftruncate;
    pub use super::init_module::init_module_1_0 as init_module;
    pub use super::newfstat::newfstat_1_0 as newfstat;
    pub use super::newlstat::newlstat_1_0 as newlstat;
    pub use super::newstat::stat_1_0 as stat;
    pub use super::setrlimit::setrlimit_1_0 as setrlimit;
    pub use super::setup::setup;
    pub use super::statfs::statfs_1_0 as statfs;
    pub use super::sysinfo::sysinfo_1_0 as sysinfo;
    pub use super::truncate::truncate_1_0 as truncate;
}

#[cfg(target_arch = "x86")]
pub use access::access;
pub use acct::acct;
pub use adjtimex::adjtimex;
#[cfg(target_arch = "x86")]
pub use alarm::alarm;
pub use brk::brk;
pub use chdir::chdir;
#[cfg(target_arch = "x86")]
pub use chmod::chmod;
pub use chroot::chroot;
pub use close::close;
#[cfg(target_arch = "x86")]
pub use creat::creat;
#[cfg(target_arch = "x86")]
pub use create_module::create_module;
pub use delete_module::delete_module;
pub use dup::dup;
#[cfg(target_arch = "x86")]
pub use dup2::dup2;
pub use execve::execve;
pub use exit::exit;
pub use fchdir::fchdir;
pub use fchmod::fchmod;
#[cfg(target_arch = "x86")]
pub use fchown::fchown16;
pub use fcntl::fcntl;
#[cfg(target_arch = "x86")]
pub use fork::fork;
#[cfg(target_arch = "x86")]
pub use fstat::oldfstat;
pub use fstatfs::fstatfs;
pub use fsync::fsync;
pub use ftruncate::ftruncate;
#[cfg(target_arch = "x86")]
pub use get_kernel_syms::get_kernel_syms;
#[cfg(target_arch = "x86")]
pub use getegid::getegid16;
#[cfg(target_arch = "x86")]
pub use geteuid::geteuid16;
#[cfg(target_arch = "x86")]
pub use getgid::getgid16;
#[cfg(target_arch = "x86")]
pub use getgroups::getgroups16;
pub use getitimer::getitimer;
pub use getpgid::getpgid;
#[cfg(target_arch = "x86")]
pub use getpgrp::getpgrp;
pub use getpid::getpid;
pub use getppid::getppid;
pub use getpriority::getpriority;
pub use getrlimit::getrlimit;
pub use getrusage::getrusage;
pub use gettimeofday::gettimeofday;
#[cfg(target_arch = "x86")]
pub use getuid::getuid16;
#[cfg(target_arch = "x86")]
pub use idle::idle;
pub use init_module::init_module;
pub use ioctl::ioctl;
#[cfg(target_arch = "x86")]
pub use ioperm::ioperm;
#[cfg(target_arch = "x86")]
pub use iopl::iopl;
#[cfg(target_arch = "x86")]
pub use ipc::{
    MSGCTL, MSGGET, MSGRCV, MSGSND, SEMCTL, SEMGET, SEMOP, SHMAT, SHMCTL,
    SHMDT, SHMGET, ipc,
};
pub use kill::kill;
#[cfg(target_arch = "x86")]
pub use lchown::lchown16;
#[cfg(target_arch = "x86")]
pub use link::link;
pub use lseek::lseek;
#[cfg(target_arch = "x86")]
pub use lstat::oldlstat;
#[cfg(target_arch = "x86")]
pub use mkdir::mkdir;
#[cfg(target_arch = "x86")]
pub use mknod::mknod;
pub use mmap::mmap;
#[cfg(target_arch = "x86")]
pub use modify_ldt::modify_ldt;
pub use mount::mount;
pub use munmap::munmap;
pub use newfstat::newfstat;
#[cfg(target_arch = "x86")]
pub use newlstat::newlstat;
#[cfg(target_arch = "x86")]
pub use newstat::stat;
#[cfg(target_arch = "x86")]
pub use nice::nice;
#[cfg(target_arch = "x86")]
pub use open::open;
#[cfg(target_arch = "x86")]
pub use pause::pause;
#[cfg(target_arch = "x86")]
pub use pipe::pipe;
pub use ptrace::ptrace;
pub use read::read;
#[cfg(target_arch = "x86")]
pub use readdir::readdir;
#[cfg(target_arch = "x86")]
pub use readlink::readlink;
pub use reboot::reboot;
#[cfg(target_arch = "x86")]
pub use rename::rename;
#[cfg(target_arch = "x86")]
pub use rmdir::rmdir;
#[cfg(target_arch = "x86")]
pub use select::select;
pub use setdomainname::setdomainname;
#[cfg(target_arch = "x86")]
pub use setgid::setgid16;
#[cfg(target_arch = "x86")]
pub use setgroups::setgroups16;
pub use sethostname::sethostname;
pub use setitimer::setitimer;
pub use setpgid::setpgid;
pub use setpriority::setpriority;
#[cfg(target_arch = "x86")]
pub use setregid::setregid16;
#[cfg(target_arch = "x86")]
pub use setreuid::setreuid16;
pub use setrlimit::setrlimit;
pub use setsid::setsid;
pub use settimeofday::settimeofday;
#[cfg(target_arch = "x86")]
pub use setuid::setuid16;
#[cfg(target_arch = "x86")]
pub use sgetmask::sgetmask;
#[cfg(target_arch = "x86")]
pub use sigaction::sigaction;
#[cfg(target_arch = "x86")]
pub use signal::{
    SIG_DFL, SIG_IGN, SigHandler, sig_handler, sig_handler_from_raw, signal,
};
#[cfg(target_arch = "x86")]
pub use sigpending::sigpending;
#[cfg(target_arch = "x86")]
pub use sigprocmask::{SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, sigprocmask};
#[cfg(target_arch = "x86")]
pub use sigreturn::sigreturn;
#[cfg(target_arch = "x86")]
pub use sigsuspend::sigsuspend;
#[cfg(target_arch = "x86")]
pub use socketcall::{
    SYS_ACCEPT, SYS_BIND, SYS_CONNECT, SYS_GETPEERNAME, SYS_GETSOCKNAME,
    SYS_GETSOCKOPT, SYS_LISTEN, SYS_RECV, SYS_RECVFROM, SYS_SEND, SYS_SENDTO,
    SYS_SETSOCKOPT, SYS_SHUTDOWN, SYS_SOCKET, SYS_SOCKETPAIR, socketcall,
};
#[cfg(target_arch = "x86")]
pub use ssetmask::ssetmask;
#[cfg(target_arch = "x86")]
pub use stat::oldstat;
pub use statfs::statfs;
#[cfg(target_arch = "x86")]
pub use stime::stime;
pub use swapoff::swapoff;
pub use swapon::swapon;
#[cfg(target_arch = "x86")]
pub use symlink::symlink;
pub use sync::sync;
pub use sysinfo::sysinfo;
pub use syslog::syslog;
#[cfg(target_arch = "x86")]
pub use time::time;
pub use times::times;
pub use truncate::truncate;
pub use umask::umask;
pub use umount::umount;
#[cfg(target_arch = "aarch64")]
pub use uname::newuname;
#[cfg(target_arch = "x86")]
pub use uname::{newuname, oldolduname, olduname};
#[cfg(target_arch = "x86")]
pub use unlink::unlink;
#[cfg(target_arch = "x86")]
pub use uselib::uselib;
#[cfg(target_arch = "x86")]
pub use ustat::ustat;
#[cfg(target_arch = "x86")]
pub use utime::utime;
pub use vhangup::vhangup;
#[cfg(target_arch = "x86")]
pub use vm86::vm86;
pub use wait4::wait4;
#[cfg(target_arch = "x86")]
pub use waitpid::waitpid;
pub use write::write;
