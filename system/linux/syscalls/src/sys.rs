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
mod adjtimex;
mod alarm;
mod brk;
mod chdir;
mod chmod;
mod chroot;
mod close;
mod creat;
mod create_module;
mod delete_module;
mod dup;
mod dup2;
mod execve;
mod exit;
mod fchdir;
mod fchmod;
mod fchown;
mod fcntl;
mod fork;
mod fstat;
mod fstatfs;
mod fsync;
mod ftruncate;
mod get_kernel_syms;
mod getegid;
mod geteuid;
mod getgid;
mod getgroups;
mod getitimer;
mod getpgid;
mod getpgrp;
mod getpid;
mod getppid;
mod getpriority;
mod getrlimit;
mod getrusage;
mod gettimeofday;
mod getuid;
mod idle;
mod init_module;
mod ioctl;
mod ioperm;
mod iopl;
mod ipc;
mod kill;
mod lchown;
mod link;
mod lseek;
mod lstat;
mod mkdir;
mod mknod;
mod mmap;
mod modify_ldt;
mod mount;
mod munmap;
mod newfstat;
mod newlstat;
mod newstat;
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
mod setdomainname;
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
mod sigprocmask;
mod sigreturn;
mod sigsuspend;
mod socketcall;
mod ssetmask;
mod stat;
mod statfs;
mod stime;
mod swapoff;
mod swapon;
mod symlink;
mod sync;
mod sysinfo;
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
mod vhangup;
mod vm86;
mod wait4;
mod waitpid;
mod write;

pub use access::access;
pub use acct::acct;
pub use adjtimex::adjtimex;
pub use alarm::alarm;
pub use brk::brk;
pub use chdir::chdir;
pub use chmod::chmod;
pub use chroot::chroot;
pub use close::close;
pub use creat::creat;
pub use create_module::create_module;
pub use delete_module::delete_module;
pub use dup::dup;
pub use dup2::dup2;
pub use execve::execve;
pub use exit::exit;
pub use fchdir::fchdir;
pub use fchmod::fchmod;
pub use fchown::fchown16;
pub use fcntl::fcntl;
pub use fork::fork;
pub use fstat::oldfstat;
pub use fstatfs::fstatfs;
pub use fsync::fsync;
pub use ftruncate::ftruncate;
pub use get_kernel_syms::get_kernel_syms;
pub use getegid::getegid16;
pub use geteuid::geteuid16;
pub use getgid::getgid16;
pub use getgroups::getgroups16;
pub use getitimer::getitimer;
pub use getpgid::getpgid;
pub use getpgrp::getpgrp;
pub use getpid::getpid;
pub use getppid::getppid;
pub use getpriority::getpriority;
pub use getrlimit::getrlimit;
pub use getrusage::getrusage;
pub use gettimeofday::gettimeofday;
pub use getuid::getuid16;
pub use idle::idle;
pub use init_module::init_module;
pub use ioctl::ioctl;
pub use ioperm::ioperm;
pub use iopl::iopl;
pub use ipc::{
    MSGCTL, MSGGET, MSGRCV, MSGSND, SEMCTL, SEMGET, SEMOP, SHMAT, SHMCTL,
    SHMDT, SHMGET, ipc,
};
pub use kill::kill;
pub use lchown::lchown16;
pub use link::link;
pub use lseek::lseek;
pub use lstat::oldlstat;
pub use mkdir::mkdir;
pub use mknod::mknod;
pub use mmap::mmap;
pub use modify_ldt::modify_ldt;
pub use mount::mount;
pub use munmap::munmap;
pub use newfstat::newfstat;
pub use newlstat::newlstat;
pub use newstat::stat;
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
pub use setdomainname::setdomainname;
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
pub use signal::{
    SIG_DFL, SIG_IGN, SigHandler, sig_handler, sig_handler_from_raw, signal,
};
pub use sigpending::sigpending;
pub use sigprocmask::{SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, sigprocmask};
pub use sigreturn::sigreturn;
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
pub use swapoff::swapoff;
pub use swapon::swapon;
pub use symlink::symlink;
pub use sync::sync;
pub use sysinfo::sysinfo;
pub use syslog::syslog;
pub use time::time;
pub use times::times;
pub use truncate::truncate;
pub use umask::umask;
pub use umount::umount;
pub use uname::{newuname, oldolduname, olduname};
pub use unlink::unlink;
pub use uselib::uselib;
pub use ustat::ustat;
pub use utime::utime;
pub use vhangup::vhangup;
pub use vm86::vm86;
pub use wait4::wait4;
pub use waitpid::waitpid;
pub use write::write;
