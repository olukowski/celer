# Progress

This checklist tracks the Linux 1.0 `include/linux/sys.h` entry points against wrappers currently exposed by this crate. A checked box means the crate wraps that syscall slot or ABI today, even if Linux 1.0 only provided a stub for it.

- `sys_clone` is a special case in Linux 1.0: `sys.h` defines `#define sys_clone sys_fork`, and the actual syscall table uses that alias at nr 120. The crate does not wrap `clone` as a separate entry today.
- Some historical names do not match the current wrapper names exactly. Those cases are called out inline.

## 0-24

- [x] `  0 sys_setup`
- [x] `  1 sys_exit`
- [x] `  2 sys_fork`
- [x] `  3 sys_read`
- [x] `  4 sys_write`
- [x] `  5 sys_open`
- [x] `  6 sys_close`
- [x] `  7 sys_waitpid`
- [x] `  8 sys_creat`
- [x] `  9 sys_link`
- [x] ` 10 sys_unlink`
- [x] ` 11 sys_execve`
- [x] ` 12 sys_chdir`
- [x] ` 13 sys_time`
- [x] ` 14 sys_mknod`
- [x] ` 15 sys_chmod`
- [x] ` 16 sys_chown` - wrapped as `lchown16`; Linux 1.0 `sys_chown` is the no-follow variant later exposed as `lchown`
- [ ] ` 17 sys_break` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 18 sys_stat` - wrapped as `oldstat`
- [x] ` 19 sys_lseek`
- [x] ` 20 sys_getpid`
- [x] ` 21 sys_mount`
- [x] ` 22 sys_umount`
- [x] ` 23 sys_setuid` - wrapped as `setuid16`
- [x] ` 24 sys_getuid` - wrapped as `getuid16`

## 25-49

- [x] ` 25 sys_stime`
- [x] ` 26 sys_ptrace`
- [x] ` 27 sys_alarm`
- [x] ` 28 sys_fstat` - wrapped as `oldfstat`
- [x] ` 29 sys_pause`
- [x] ` 30 sys_utime`
- [ ] ` 31 sys_stty` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [ ] ` 32 sys_gtty` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 33 sys_access`
- [x] ` 34 sys_nice`
- [ ] ` 35 sys_ftime` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 36 sys_sync`
- [x] ` 37 sys_kill`
- [x] ` 38 sys_rename`
- [x] ` 39 sys_mkdir`
- [x] ` 40 sys_rmdir`
- [x] ` 41 sys_dup`
- [x] ` 42 sys_pipe`
- [x] ` 43 sys_times`
- [ ] ` 44 sys_prof` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 45 sys_brk`
- [x] ` 46 sys_setgid` - wrapped as `setgid16`
- [x] ` 47 sys_getgid` - wrapped as `getgid16`
- [x] ` 48 sys_signal`
- [x] ` 49 sys_geteuid` - wrapped as `geteuid16`

## 50-74

- [x] ` 50 sys_getegid` - wrapped as `getegid16`
- [x] ` 51 sys_acct` - wrapped today, but stubbed in Linux 1.0: always returns `-ENOSYS`
- [ ] ` 52 sys_phys` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [ ] ` 53 sys_lock` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 54 sys_ioctl`
- [x] ` 55 sys_fcntl`
- [ ] ` 56 sys_mpx` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 57 sys_setpgid`
- [ ] ` 58 sys_ulimit` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 59 sys_uname` - wrapped as `oldolduname`; the actual Linux 1.0 syscall entry for nr 59 is the legacy uname ABI
- [x] ` 60 sys_umask`
- [x] ` 61 sys_chroot`
- [x] ` 62 sys_ustat` - wrapped today, but stubbed in Linux 1.0: always returns `-ENOSYS`
- [x] ` 63 sys_dup2`
- [x] ` 64 sys_getppid`
- [x] ` 65 sys_getpgrp`
- [x] ` 66 sys_setsid`
- [x] ` 67 sys_sigaction`
- [x] ` 68 sys_sgetmask`
- [x] ` 69 sys_ssetmask`
- [x] ` 70 sys_setreuid` - wrapped as `setreuid16`
- [x] ` 71 sys_setregid` - wrapped as `setregid16`
- [x] ` 72 sys_sigsuspend`
- [x] ` 73 sys_sigpending`
- [x] ` 74 sys_sethostname`

## 75-99

- [ ] ` 75 sys_setrlimit`
- [x] ` 76 sys_getrlimit`
- [x] ` 77 sys_getrusage`
- [ ] ` 78 sys_gettimeofday`
- [ ] ` 79 sys_settimeofday`
- [ ] ` 80 sys_getgroups`
- [ ] ` 81 sys_setgroups`
- [ ] ` 82 sys_select`
- [ ] ` 83 sys_symlink`
- [ ] ` 84 sys_lstat`
- [ ] ` 85 sys_readlink`
- [ ] ` 86 sys_uselib`
- [ ] ` 87 sys_swapon`
- [ ] ` 88 sys_reboot`
- [ ] ` 89 sys_readdir`
- [ ] ` 90 sys_mmap`
- [ ] ` 91 sys_munmap`
- [ ] ` 92 sys_truncate`
- [ ] ` 93 sys_ftruncate`
- [ ] ` 94 sys_fchmod`
- [ ] ` 95 sys_fchown`
- [ ] ` 96 sys_getpriority`
- [ ] ` 97 sys_setpriority`
- [ ] ` 98 sys_profil` - stubbed in Linux 1.0: always returns `-ENOSYS`
- [ ] ` 99 sys_statfs`

## 100-124

- [ ] `100 sys_fstatfs`
- [ ] `101 sys_ioperm`
- [ ] `102 sys_socketcall`
- [ ] `103 sys_syslog`
- [ ] `104 sys_getitimer`
- [ ] `105 sys_setitimer`
- [ ] `106 sys_newstat`
- [ ] `107 sys_newlstat`
- [ ] `108 sys_newfstat`
- [ ] `109 sys_newuname`
- [ ] `110 sys_iopl`
- [ ] `111 sys_vhangup`
- [ ] `112 sys_idle`
- [ ] `113 sys_vm86`
- [ ] `114 sys_wait4`
- [ ] `115 sys_swapoff`
- [ ] `116 sys_sysinfo`
- [ ] `117 sys_ipc`
- [ ] `118 sys_fsync`
- [ ] `119 sys_sigreturn`
- [ ] `120 sys_setdomainname`
- [ ] `121 sys_olduname` - historical legacy uname entry; wrapped today as `oldolduname` via nr 59, not as a separate newer uname variant
- [ ] `122 sys_old_syscall` - stubbed in Linux 1.0: `sys_old_syscall()` always returns `-ENOSYS`
- [ ] `123 sys_modify_ldt`
- [ ] `124 sys_adjtimex`

## 125-134

- [ ] `125 sys_mprotect` - present in Linux 1.0, but explicitly marked not implemented yet: always returns `-EINVAL`
- [ ] `126 sys_sigprocmask`
- [ ] `127 sys_create_module`
- [ ] `128 sys_init_module`
- [ ] `129 sys_delete_module`
- [ ] `130 sys_get_kernel_syms`
- [ ] `131 sys_quotactl` - placeholder in Linux 1.0: mapped to `sys_ni_syscall` in `sys.h`
- [ ] `132 sys_getpgid`
- [ ] `133 sys_fchdir`
- [ ] `134 sys_bdflush` - placeholder in Linux 1.0: mapped to `sys_ni_syscall` in `sys.h`
