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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod getpgrp;
mod getpid;
mod getppid;
mod getpriority;
mod getrlimit;
mod getrusage;
mod gettimeofday;
#[cfg(target_arch = "x86")]
mod getuid;
mod helpers;
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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod link;
mod lseek;
#[cfg(target_arch = "x86")]
mod lstat;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod mkdir;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod mknod;
mod mmap;
#[cfg(target_arch = "x86")]
mod modify_ldt;
mod mount;
mod munmap;
mod newfstat;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod newlstat;
#[cfg(target_arch = "x86")]
mod nice;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod open;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod pause;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod pipe;
mod ptrace;
mod read;
#[cfg(target_arch = "x86")]
mod readdir;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod readlink;
mod reboot;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod rename;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod rmdir;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod stat;
mod statfs;
pub mod sys;
mod uname;

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
#[cfg(target_arch = "x86")]
pub use fstat::{OldfstatError, oldfstat};
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
pub use getpgid::{GetpgidError, getpgid};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use getpgrp::getpgrp;
pub use getpid::getpid;
pub use getppid::getppid;
pub use getpriority::{GetpriorityError, getpriority};
pub use getrlimit::{GetrlimitError, getrlimit};
pub use getrusage::{GetrusageError, getrusage};
pub use gettimeofday::{GettimeofdayError, gettimeofday};
#[cfg(target_arch = "x86")]
pub use getuid::getuid16;
#[cfg(target_arch = "x86")]
pub use idle::{IdleError, idle};
pub use init_module::{InitModuleError, init_module};
pub use ioctl::{IoctlError, ioctl};
#[cfg(target_arch = "x86")]
pub use ioperm::{IopermError, ioperm};
#[cfg(target_arch = "x86")]
pub use iopl::{IoplError, iopl};
#[cfg(target_arch = "x86")]
pub use ipc::{
    IpcError, MSGCTL, MSGGET, MSGRCV, MSGSND, SEMCTL, SEMGET, SEMOP, SHMAT,
    SHMCTL, SHMDT, SHMGET, ipc,
};
pub use kill::{KillError, kill};
#[cfg(target_arch = "x86")]
pub use lchown::{Lchown16Error, lchown16};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use link::{LinkError, link};
pub use lseek::{LseekError, lseek};
#[cfg(target_arch = "x86")]
pub use lstat::{OldlstatError, oldlstat};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use mkdir::{MkdirError, mkdir};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use mknod::{MknodError, mknod};
pub use mmap::{MmapError, mmap};
#[cfg(target_arch = "x86")]
pub use modify_ldt::{ModifyLdtError, modify_ldt};
pub use mount::{MountError, mount};
pub use munmap::{MunmapError, munmap};
pub use newfstat::{NewfstatError, newfstat};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use newlstat::{NewlstatError, newlstat};
#[cfg(target_arch = "x86")]
pub use nice::{NiceError, nice};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use open::{OpenError, open};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use pause::{PauseError, pause};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use pipe::{PipeError, pipe};
pub use ptrace::{PtraceError, ptrace};
pub use read::{ReadError, read};
#[cfg(target_arch = "x86")]
pub use readdir::{ReaddirError, readdir};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use readlink::{ReadlinkError, readlink};
pub use reboot::{RebootError, reboot};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use rename::{RenameError, rename};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use rmdir::{RmdirError, rmdir};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use select::{SelectError, select};
pub use setdomainname::{SetdomainnameError, setdomainname};
#[cfg(target_arch = "x86")]
pub use setgid::{Setgid16Error, setgid16};
#[cfg(target_arch = "x86")]
pub use setgroups::{Setgroups16Error, setgroups16};
pub use sethostname::{SethostnameError, sethostname};
pub use setitimer::{SetitimerError, setitimer};
pub use setpgid::{SetpgidError, setpgid};
pub use setpriority::{SetpriorityError, setpriority};
#[cfg(target_arch = "x86")]
pub use setregid::{Setregid16Error, setregid16};
#[cfg(target_arch = "x86")]
pub use setreuid::{Setreuid16Error, setreuid16};
pub use setrlimit::{SetrlimitError, setrlimit};
pub use setsid::{SetsidError, setsid};
pub use settimeofday::{SettimeofdayError, settimeofday};
#[cfg(target_arch = "x86")]
pub use setuid::{Setuid16Error, setuid16};
#[cfg(target_arch = "x86")]
pub use sgetmask::{SgetmaskError, sgetmask};
#[cfg(target_arch = "x86")]
pub use sigaction::{SigactionError, sigaction};
#[cfg(target_arch = "x86")]
pub use signal::{
    SIG_DFL, SIG_IGN, SigHandler, SignalError, sig_handler, signal,
};
#[cfg(target_arch = "x86")]
pub use sigpending::{SigpendingError, sigpending};
#[cfg(target_arch = "x86")]
pub use sigprocmask::{
    SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, SigprocmaskError, sigprocmask,
};
#[cfg(target_arch = "x86")]
pub use sigreturn::sigreturn;
#[cfg(target_arch = "x86")]
pub use sigsuspend::{SigsuspendError, sigsuspend};
#[cfg(target_arch = "x86")]
pub use socketcall::{
    SYS_ACCEPT, SYS_BIND, SYS_CONNECT, SYS_GETPEERNAME, SYS_GETSOCKNAME,
    SYS_GETSOCKOPT, SYS_LISTEN, SYS_RECV, SYS_RECVFROM, SYS_SEND, SYS_SENDTO,
    SYS_SETSOCKOPT, SYS_SHUTDOWN, SYS_SOCKET, SYS_SOCKETPAIR, SocketcallError,
    socketcall,
};
#[cfg(target_arch = "x86")]
pub use ssetmask::{SsetmaskError, ssetmask};
#[cfg(target_arch = "x86")]
pub use stat::OldstatError;
#[cfg(target_arch = "x86")]
pub use stat::oldstat;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use stat::{StatError, stat};
pub use statfs::{StatfsError, statfs};
pub use uname::{NewunameError, newuname};
#[cfg(target_arch = "x86")]
pub use uname::{OldoldunameError, OldunameError, oldolduname, olduname};

/// Wrapped historical Linux 1.0 syscall ABIs.
#[cfg(target_arch = "x86")]
pub mod linux_1_0 {
    pub use super::create_module::{CreateModuleError, create_module};
    pub use super::fstatfs::{FstatfsError, fstatfs_1_0 as fstatfs};
    pub use super::ftruncate::{
        Ftruncate1_0Error as FtruncateError, ftruncate_1_0 as ftruncate,
    };
    pub use super::newfstat::{NewfstatError, newfstat_1_0 as newfstat};
    pub use super::newlstat::{NewlstatError, newlstat_1_0 as newlstat};
    pub use super::setrlimit::{SetrlimitError, setrlimit_1_0 as setrlimit};
    pub use super::stat::{StatError, stat_1_0 as stat};
    pub use super::statfs::{StatfsError, statfs_1_0 as statfs};
}
