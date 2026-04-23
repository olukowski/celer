use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall3};

/// Replace the current process image with a new program.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes:
///   - Linux 1.0 copied the filename with `getname()`, then opened and
///     validated the target directly in `do_execve()`.
///   - Current kernels route `execve` through `do_execveat_common()`, which
///     counts `argv` / `envp`, enforces stack limits up front, inserts an
///     empty `argv[0]` when the caller supplies no arguments, and then runs
///     the binary-format search path.
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - If successful, this call does not return to the current program image.
/// - `filename` names the program to execute.
/// - `argv` and `envp` point to null-terminated arrays of null-terminated strings.
/// - On current kernels, `argv == NULL` or an empty argument vector is turned
///   into a single empty `argv[0]` entry before the binary runs.
/// # Errors
/// - `EAGAIN`: on current kernels, `PF_NPROC_EXCEEDED` is set and the caller
///   is still over `RLIMIT_NPROC`.
/// - `EFAULT`: the filename pointer, one of the `argv` / `envp` vector
///   pointers, or one of the pointed-to strings is not accessible.
/// - `E2BIG`: the argument or environment vectors exceed the counted-string or
///   stack-space limits enforced before `bprm_execve()`.
/// - `ENOEXEC`: no registered binary handler accepts the target image.
/// - `ELOOP`: current kernels exceed the interpreter / binfmt rewrite limit.
/// - Other reachable lookup, permission, allocation, and binary-handler
///   errors are returned by `getname()`, `alloc_bprm()` / `open_namei()`, and
///   the binary-format loaders.
///
/// # References
/// - Stable entry:
///   [v7.0 execve](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/exec.c?h=v7.0#n1924)
/// - Stable helper:
///   [v7.0 do_execveat_common](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/exec.c?h=v7.0#n1778)
/// - Stable binary search:
///   [v7.0 search_binary_handler](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/exec.c?h=v7.0#n1645)
/// - LTS entry:
///   [v6.18.18 execve](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/exec.c?h=v6.18.18#n2005)
/// - LTS helper:
///   [v6.18.18 do_execveat_common](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/exec.c?h=v6.18.18#n1784)
/// - LTS binary search:
///   [v6.18.18 search_binary_handler](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/exec.c?h=v6.18.18#n1651)
/// - First stable:
///   [Linux 1.0 sys_execve](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/exec.c?h=1.0#n711)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.10 syscall table entry](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/system_call.s?h=0.10#n154)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn execve(
    filename: *const Char,
    argv: *const *const Char,
    envp: *const *const Char,
) -> Long {
    // SAFETY: this wrapper forwards the raw user pointers without
    // dereferencing them in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    (unsafe {
        syscall3(
            Sysno::Execve,
            filename.addr() as isize,
            argv.addr() as isize,
            envp.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{Char, Int, PidT};

    use super::execve;
    use crate::sys::{exit, fork, waitpid};

    #[test]
    fn test_execve() {
        let pid = fork();
        fn use_pid(pid: PidT) {
            if pid == 0 {
                let filename = b"/bin/true\0";
                let argv: [*const Char; 2] =
                    [filename.as_ptr().cast(), core::ptr::null()];
                let envp: [*const Char; 1] = [core::ptr::null()];

                let ret = execve(
                    filename.as_ptr().cast(),
                    argv.as_ptr(),
                    envp.as_ptr(),
                );

                if ret < 0 {
                    exit(1);
                }
                exit(0);
            }
        }

        use_pid(pid);

        let mut status: Int = 0;
        let waited = unsafe { waitpid(pid, &mut status, 0) };

        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }
}
