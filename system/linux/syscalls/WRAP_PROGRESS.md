# Wrap Progress

This checklist tracks the next layer above the raw Linux syscall wrappers.
The goal of this layer is to preserve the raw syscall shape while removing
the most mechanical Rust rough edges.

## Scope

The wrap layer should:

- convert raw syscall return values into `Result<T, E>`;
- use syscall-specific error enums, with an `Other(Errno)` escape hatch when
  the syscall can delegate to file systems, drivers, protocols, security
  modules, or other helpers with wider reachable errors;
- use shared slices for `*const T` plus length arguments;
- use mutable slices for `*mut T` plus length arguments when the kernel reads
  and writes initialized elements;
- use `MaybeUninit` for output buffers or output slots that the kernel
  initializes;
- use shared references for non-null single-item `*const T` arguments;
- use mutable references for non-null single-item `*mut T` arguments when the
  pointed-to value must already be initialized;
- use `Option<&T>`, `Option<&mut T>`, or `Option<&mut MaybeUninit<T>>` when
  null is a meaningful syscall input;
- use `&CStr` for NUL-terminated `*const Char` strings;
- stay safe whenever the wrapper removes all caller-visible memory-safety
  preconditions;
- use strict-provenance APIs when converting pointers before passing raw
  integer syscall arguments.

The wrap layer should not introduce ownership, buffering, retry policy, path
encoding policy, automatic descriptor closing, or broad semantic convenience.
Those belong in higher layers.

## Module Layout

The existing `sys` module remains the raw syscall layer. Its wrappers keep the
exact kernel-facing ABI shape: raw pointer arguments, raw integer return values,
and `unsafe` where the caller must uphold memory-safety preconditions.

The crate root exports the wrap layer. `lib.rs` should mirror the shape of
`sys.rs`: declare per-syscall wrapper modules and re-export each public wrapper
and its syscall-specific error type from the crate root. For example,
`celer_system_linux_syscalls::read` is the wrapped API, while
`celer_system_linux_syscalls::sys::read` is the raw API.

Architecture availability should mirror the raw layer. Top-level wrapped
modules and re-exports must use the same kind of `cfg` gates as the raw
syscalls so only syscalls available on the current target are exposed.

Historical Linux 1.0 ABI wrappers should be exposed from a top-level
`linux_1_0` module, mirroring `sys::linux_1_0`.

## Errno

The shared errno representation should be an enum. Common errno values should
be listed explicitly, and a raw variant should carry any errno value that the
crate does not list explicitly.

The raw variant should hold a `u16` value. When a raw syscall return is in the
kernel errno range `-4095..=-1`, the stored errno value is the negated kernel
return value.

Syscall-specific error enums should use explicit variants for source-verified
reachable errors. They should include an `Other(Errno)` variant when the syscall
can return delegated, object-specific, or otherwise open-ended errors.

## Done Criteria

A checked syscall means:

- the wrapper exists and is public from the wrap layer;
- each pointer argument has been classified as input, output, in/out, nullable,
  or NUL-terminated string from kernel source;
- each safe/unsafe choice follows the repository unsafe rule;
- the success return type is semantic for this syscall;
- reachable direct errors are represented in a syscall-specific error type;
- delegated or object-specific errors use `Other(Errno)` unless the reachable
  set is source-verified to be closed;
- tests cover success and at least one practical error path when feasible.

## Documentation

Wrapped syscall docs should document wrapper semantics, not kernel behavior.
Kernel support, historical notes, required privileges, detailed behavior,
reachable error analysis, and source references belong in `sys::*`.

Each wrapped syscall should document:

- the Rust argument conversions it provides;
- the `Ok(...)` value when it is not obvious;
- the syscall-specific error variants;
- any remaining caller-visible safety precondition;
- a link to the raw `sys::*` wrapper for kernel behavior, reachable errors,
  and source references.

## Helpers

Shared conversion helpers should live in a private `helpers` module.

## Checklist

- [x] `access` - x86, x86_64
- [x] `acct`
- [x] `adjtimex`
- [x] `alarm` - x86, x86_64
- [x] `brk`
- [x] `chdir`
- [x] `chmod` - x86, x86_64
- [x] `chroot`
- [x] `close`
- [x] `creat` - x86, x86_64
- [x] `create_module` - x86
- [x] `delete_module`
- [x] `dup`
- [x] `dup2` - x86, x86_64
- [x] `execve`
- [x] `exit`
- [x] `fchdir`
- [x] `fchmod`
- [x] `fchown16` - x86
- [x] `fcntl`
- [x] `fork` - x86
- [x] `fstatfs`
- [x] `fsync`
- [x] `ftruncate`
- [x] `get_kernel_syms` - x86
- [x] `getegid16` - x86
- [x] `geteuid16` - x86
- [x] `getgid16` - x86
- [x] `getgroups16` - x86
- [x] `getitimer`
- [x] `getpgid`
- [x] `getpgrp` - x86, x86_64
- [x] `getpid`
- [x] `getppid`
- [x] `getpriority`
- [x] `getrlimit`
- [x] `getrusage`
- [x] `gettimeofday`
- [x] `getuid16` - x86
- [x] `idle` - x86
- [x] `init_module`
- [x] `ioctl`
- [x] `ioperm` - x86
- [x] `iopl` - x86
- [x] `ipc` - x86
- [x] `kill`
- [x] `lchown16` - x86
- [x] `link` - x86, x86_64
- [x] `lseek`
- [x] `mknod` - x86, x86_64
- [x] `mkdir` - x86, x86_64
- [x] `mmap`
- [x] `modify_ldt` - x86
- [x] `mount`
- [x] `munmap`
- [x] `newfstat`
- [x] `newlstat` - x86, x86_64
- [x] `newuname`
- [x] `nice` - x86
- [x] `oldfstat` - x86
- [x] `oldlstat` - x86
- [x] `oldolduname` - x86
- [ ] `oldstat` - x86
- [ ] `olduname` - x86
- [ ] `open` - x86, x86_64
- [ ] `pause` - x86, x86_64
- [ ] `pipe` - x86, x86_64
- [ ] `ptrace`
- [ ] `read`
- [ ] `readdir` - x86
- [ ] `readlink` - x86, x86_64
- [ ] `reboot`
- [ ] `rename` - x86, x86_64
- [ ] `rmdir` - x86, x86_64
- [ ] `select` - x86, x86_64
- [ ] `setdomainname`
- [ ] `setgid16` - x86
- [ ] `setgroups16` - x86
- [ ] `sethostname`
- [ ] `setitimer`
- [ ] `setpgid`
- [ ] `setpriority`
- [ ] `setregid16` - x86
- [ ] `setreuid16` - x86
- [ ] `setrlimit`
- [ ] `setsid`
- [ ] `settimeofday`
- [ ] `setuid16` - x86
- [ ] `sgetmask` - x86
- [ ] `sigaction` - x86
- [ ] `signal` - x86
- [ ] `sigpending` - x86
- [ ] `sigprocmask` - x86
- [ ] `sigreturn` - x86
- [ ] `sigsuspend` - x86
- [ ] `socketcall` - x86
- [ ] `ssetmask` - x86
- [ ] `stat` - x86, x86_64
- [ ] `statfs`
- [ ] `stime` - x86
- [ ] `swapoff`
- [ ] `swapon`
- [ ] `symlink` - x86, x86_64
- [ ] `sync`
- [ ] `sysinfo`
- [ ] `syslog`
- [ ] `time` - x86, x86_64
- [ ] `times`
- [ ] `truncate`
- [ ] `umask`
- [ ] `umount`
- [ ] `unlink` - x86, x86_64
- [ ] `uselib` - x86
- [ ] `ustat` - x86, x86_64
- [ ] `utime` - x86, x86_64
- [ ] `vhangup`
- [ ] `vm86` - x86
- [ ] `wait4`
- [ ] `waitpid` - x86
- [ ] `write`

## Linux 1.0 Historical Variants

These raw wrappers are currently exposed under `sys::linux_1_0` on x86. Decide
whether the wrap layer mirrors them before checking them off.

- [x] `linux_1_0::fstatfs`
- [x] `linux_1_0::ftruncate`
- [ ] `linux_1_0::init_module`
- [ ] `linux_1_0::newfstat`
- [ ] `linux_1_0::newlstat`
- [ ] `linux_1_0::setrlimit`
- [ ] `linux_1_0::setup`
- [ ] `linux_1_0::stat`
- [ ] `linux_1_0::statfs`
- [ ] `linux_1_0::sysinfo`
- [ ] `linux_1_0::truncate`
