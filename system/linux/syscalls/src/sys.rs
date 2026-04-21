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
mod close;
mod creat;
mod dup;
mod execve;
mod exit;
mod fork;
mod fstat;
mod getegid;
mod geteuid;
mod getgid;
mod getpid;
mod getuid;
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
mod setuid;
mod signal;
mod stat;
mod stime;
mod sync;
mod time;
mod times;
mod umount;
mod unlink;
mod utime;
mod waitpid;
mod write;

pub use access::access;
pub use acct::acct;
pub use alarm::alarm;
pub use brk::brk;
pub use chdir::chdir;
pub use chmod::chmod;
pub use close::close;
pub use creat::creat;
pub use dup::dup;
pub use execve::execve;
pub use exit::exit;
pub use fork::fork;
pub use fstat::fstat;
pub use getegid::getegid;
pub use geteuid::geteuid;
pub use getgid::getgid;
pub use getpid::getpid;
pub use getuid::getuid;
pub use kill::kill;
pub use lchown::lchown;
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
pub use setgid::setgid;
pub use setuid::setuid;
pub use signal::signal;
pub use stat::stat;
pub use stime::stime;
pub use sync::sync;
pub use time::time;
pub use times::times;
pub use umount::umount;
pub use unlink::unlink;
pub use utime::utime;
pub use waitpid::waitpid;
pub use write::write;
