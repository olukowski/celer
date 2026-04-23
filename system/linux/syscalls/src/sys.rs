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
mod fchmod;
mod fchown;
mod fcntl;
mod fork;
mod fstat;
mod fstatfs;
mod ftruncate;
mod getegid;
mod geteuid;
mod getgid;
mod getgroups;
mod getpgrp;
mod getpid;
mod getppid;
mod getpriority;
mod getrlimit;
mod getrusage;
mod gettimeofday;
mod getuid;
mod ioctl;
mod ioperm;
mod kill;
mod lchown;
mod link;
mod lseek;
mod lstat;
mod mkdir;
mod mknod;
mod mmap;
mod mount;
mod munmap;
mod nice;
mod open;
mod pause;
mod pipe;
mod ptrace;
mod read;
mod readdir;
mod readlink;
mod reboot;
mod rename;
mod rmdir;
mod select;
mod setgid;
mod setgroups;
mod sethostname;
mod setitimer;
mod setpgid;
mod setpriority;
mod setregid;
mod setreuid;
mod setrlimit;
mod setsid;
mod settimeofday;
mod setuid;
mod setup;
mod sgetmask;
mod sigaction;
mod signal;
mod sigpending;
mod sigsuspend;
mod socketcall;
mod ssetmask;
mod stat;
mod statfs;
mod stime;
mod swapon;
mod symlink;
mod sync;
mod syslog;
mod time;
mod times;
mod truncate;
mod umask;
mod umount;
mod uname;
mod unlink;
mod uselib;
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
pub use fchmod::fchmod;
pub use fchown::fchown16;
pub use fcntl::fcntl;
pub use fork::fork;
pub use fstat::oldfstat;
pub use fstatfs::fstatfs;
pub use ftruncate::ftruncate;
pub use getegid::getegid16;
pub use geteuid::geteuid16;
pub use getgid::getgid16;
pub use getgroups::getgroups16;
pub use getpgrp::getpgrp;
pub use getpid::getpid;
pub use getppid::getppid;
pub use getpriority::getpriority;
pub use getrlimit::getrlimit;
pub use getrusage::getrusage;
pub use gettimeofday::gettimeofday;
pub use getuid::getuid16;
pub use ioctl::ioctl;
pub use ioperm::ioperm;
pub use kill::kill;
pub use lchown::lchown16;
pub use link::link;
pub use lseek::lseek;
pub use lstat::oldlstat;
pub use mkdir::mkdir;
pub use mknod::mknod;
pub use mmap::mmap;
pub use mount::mount;
pub use munmap::munmap;
pub use nice::nice;
pub use open::open;
pub use pause::pause;
pub use pipe::pipe;
pub use ptrace::ptrace;
pub use read::read;
pub use readdir::readdir;
pub use readlink::readlink;
pub use reboot::reboot;
pub use rename::rename;
pub use rmdir::rmdir;
pub use select::select;
pub use setgid::setgid16;
pub use setgroups::setgroups16;
pub use sethostname::sethostname;
pub use setitimer::setitimer;
pub use setpgid::setpgid;
pub use setpriority::setpriority;
pub use setregid::setregid16;
pub use setreuid::setreuid16;
pub use setrlimit::setrlimit;
pub use setsid::setsid;
pub use settimeofday::settimeofday;
pub use setuid::setuid16;
pub use setup::setup;
pub use sgetmask::sgetmask;
pub use sigaction::sigaction;
pub use signal::signal;
pub use sigpending::sigpending;
pub use sigsuspend::sigsuspend;
pub use socketcall::{
    SYS_ACCEPT, SYS_BIND, SYS_CONNECT, SYS_GETPEERNAME, SYS_GETSOCKNAME,
    SYS_GETSOCKOPT, SYS_LISTEN, SYS_RECV, SYS_RECVFROM, SYS_SEND, SYS_SENDTO,
    SYS_SETSOCKOPT, SYS_SHUTDOWN, SYS_SOCKET, SYS_SOCKETPAIR, socketcall,
};
pub use ssetmask::ssetmask;
pub use stat::oldstat;
pub use statfs::statfs;
pub use stime::stime;
pub use swapon::swapon;
pub use symlink::symlink;
pub use sync::sync;
pub use syslog::syslog;
pub use time::time;
pub use times::times;
pub use truncate::truncate;
pub use umask::umask;
pub use umount::umount;
pub use uname::oldolduname;
pub use unlink::unlink;
pub use uselib::uselib;
pub use ustat::ustat;
pub use utime::utime;
pub use waitpid::waitpid;
pub use write::write;
