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
//! the caller is responsible for interpreting them.
//! Note: there are some system calls that cannot fail.

mod chdir;
mod chmod;
mod close;
mod creat;
mod execve;
mod exit;
mod fork;
mod getpid;
mod getuid;
mod lchown;
mod link;
mod lseek;
mod mknod;
mod mount;
mod open;
mod ptrace;
mod read;
mod setuid;
mod stat;
mod stime;
mod time;
mod umount;
mod unlink;
mod waitpid;
mod write;

pub use chdir::chdir;
pub use chmod::chmod;
pub use close::close;
pub use creat::creat;
pub use execve::execve;
pub use exit::exit;
pub use fork::fork;
pub use getpid::getpid;
pub use getuid::getuid;
pub use lchown::lchown;
pub use link::link;
pub use lseek::lseek;
pub use mknod::mknod;
pub use mount::mount;
pub use open::open;
pub use ptrace::ptrace;
pub use read::read;
pub use setuid::setuid;
pub use stat::stat;
pub use stime::stime;
pub use time::time;
pub use umount::umount;
pub use unlink::unlink;
pub use waitpid::waitpid;
pub use write::write;
