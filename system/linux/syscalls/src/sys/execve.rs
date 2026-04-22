use celer_system_linux_ctypes::{Char, Long};

use crate::arch::current::{Sysno, syscall3};

/// Replace the current process image with a new program.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - If successful, this call does not return to the current program image.
/// - `filename` names the program to execute.
/// - `argv` and `envp` point to null-terminated arrays of null-terminated strings.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/execve.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/exec.c?h=v6.19#n2004)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/fs/exec.c?h=v6.18.18#n2005)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/fs/exec.c?h=1.0#n709)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/system_call.s?h=0.10#n154)
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
mod tests {
    use celer_system_linux_ctypes::{Char, Int, PidT};

    use super::execve;
    use crate::sys::{exit, fork, waitpid};

    #[test]
    fn test_execve() {
        let pid = fork();

        #[cfg_attr(coverage_nightly, coverage(off))] // llvm-cov can't track across the `fork` boundary
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
