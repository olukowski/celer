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
mod close;
mod creat;
mod execve;
mod exit;
mod fork;
mod getpid;
mod link;
mod open;
mod read;
mod unlink;
mod waitpid;
mod write;

pub use chdir::chdir;
pub use close::close;
pub use creat::creat;
pub use execve::execve;
pub use exit::exit;
pub use fork::fork;
pub use getpid::getpid;
pub use link::link;
pub use open::open;
pub use read::read;
pub use unlink::unlink;
pub use waitpid::waitpid;
pub use write::write;
