//! Linux system calls.
//!
//! Each system call documents its own:
//! - Behavior
//! - Kernel support
//! - Required privileges
//! - Errors
//!
//! Also, each system call has references to the "latest" source code of the
//! system call in the Linux kernel. The exact definiton of "latest" here
//! is: the latest version at the time of writing.
//!
//! The return value of a system call is the raw kernel return value.
//! Negative values in the range `[-4095, -1]` indicate errno codes;
//! the caller is responsible for interpreting them, except for syscalls such
//! as `brk` that return raw address values instead of signed errno-shaped
//! values.
//! Note: there are some system calls that cannot fail.

mod access;
mod acct;
mod alarm;
mod brk;
mod chdir;
mod chmod;
mod chroot;
mod close;
mod creat;
mod dup;
mod dup2;
mod execve;
mod exit;
mod fcntl;
mod fork;
mod fstat;
mod getegid;
mod geteuid;
mod getgid;
mod getpgrp;
mod getpid;
mod getppid;
mod getuid;
mod ioctl;
mod kill;
mod lchown;
mod link;
mod lseek;
mod mkdir;
mod mknod;
mod mount;
mod nice;
mod open;
mod pause;
mod pipe;
mod ptrace;
mod read;
mod rename;
mod rmdir;
mod setgid;
mod setpgid;
mod setregid;
mod setreuid;
mod setsid;
mod setuid;
mod sgetmask;
mod sigaction;
mod signal;
mod ssetmask;
mod stat;
mod stime;
mod sync;
mod time;
mod times;
mod umask;
mod umount;
mod uname;
mod unlink;
mod ustat;
mod utime;
mod waitpid;
mod write;

pub use access::access;
pub use acct::acct;
pub use alarm::alarm;
pub use brk::brk;
pub use chdir::chdir;
pub use chmod::chmod;
pub use chroot::chroot;
pub use close::close;
pub use creat::creat;
pub use dup::dup;
pub use dup2::dup2;
pub use execve::execve;
pub use exit::exit;
pub use fcntl::fcntl;
pub use fork::fork;
pub use fstat::oldfstat;
pub use getegid::getegid16;
pub use geteuid::geteuid16;
pub use getgid::getgid16;
pub use getpgrp::getpgrp;
pub use getpid::getpid;
pub use getppid::getppid;
pub use getuid::getuid16;
pub use ioctl::ioctl;
pub use kill::kill;
pub use lchown::lchown16;
pub use link::link;
pub use lseek::lseek;
pub use mkdir::mkdir;
pub use mknod::mknod;
pub use mount::mount;
pub use nice::nice;
pub use open::open;
pub use pause::pause;
pub use pipe::pipe;
pub use ptrace::ptrace;
pub use read::read;
pub use rename::rename;
pub use rmdir::rmdir;
pub use setgid::setgid16;
pub use setpgid::setpgid;
pub use setregid::setregid16;
pub use setreuid::setreuid16;
pub use setsid::setsid;
pub use setuid::setuid16;
pub use sgetmask::sgetmask;
pub use sigaction::sigaction;
pub use signal::signal;
pub use ssetmask::ssetmask;
pub use stat::oldstat;
pub use stime::stime;
pub use sync::sync;
pub use time::time;
pub use times::times;
pub use umask::umask;
pub use umount::umount;
pub use uname::oldolduname;
pub use unlink::unlink;
pub use ustat::ustat;
pub use utime::utime;
pub use waitpid::waitpid;
pub use write::write;
