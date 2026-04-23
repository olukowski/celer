use celer_system_linux_ctypes::{Int, Void};

use crate::arch::current::{Sysno, syscall4};

/// Reboot the system or toggle Ctrl-Alt-Del handling.
///
/// This wrapper targets the original Linux 1.0 syscall slot while also
/// exposing the modern fourth argument. Linux 1.0 reads only the first three
/// syscall arguments, so the extra `arg` register slot is ignored there.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepted only hard reset and Ctrl-Alt-Del
///   toggling commands; current kernels keep the historical magic checks but
///   add newer commands such as halt, power-off, restart2, kexec, and software
///   suspend
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - Linux 1.0: the caller must be the superuser.
/// - Current kernels: the caller must have `CAP_SYS_BOOT` in the active PID
///   namespace's user namespace.
///
/// # Behavior
/// - Linux 1.0 requires `magic == 0xfee1dead` and
///   `magic_too == 672274793`.
/// - Current kernels require `magic == 0xfee1dead` and accept
///   `magic_too == 672274793`, `85072278`, `369367448`, or `537993216`.
/// - Linux 1.0 accepts only these `cmd` values:
///   `0x01234567` for immediate hard reset, `0x89ABCDEF` to make
///   Ctrl-Alt-Del reboot, and `0` to make Ctrl-Alt-Del send `SIGINT` to init.
/// - Current kernels still accept those historical command values and add
///   newer commands at the same syscall entry.
/// - Linux 1.0 ignores `arg`.
/// - On current kernels, `arg` is consulted only for
///   `LINUX_REBOOT_CMD_RESTART2`; other commands ignore it at the syscall
///   entry.
/// - Successful hard-reset, halt, power-off, kexec, and suspend commands may
///   not return.
/// - Returns `0` on success when the chosen command returns normally.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to use the reboot syscall, or a
///   current-kernel software-suspend request reaches `hibernate()` when
///   hibernation is unavailable.
/// - `EINVAL`: the magic values are wrong, or the command is unsupported for
///   the running kernel and namespace.
/// - `EFAULT`: on current kernels, `cmd == 0xA1B2C3D4`
///   (`LINUX_REBOOT_CMD_RESTART2`) after the call reaches the init PID
///   namespace's main reboot dispatch and `arg` does not point to a readable
///   user string.
/// - Current kernels may also return `EBUSY`, `EAGAIN`, or `EOPNOTSUPP` for
///   newer commands such as kexec or software suspend.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/reboot.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/reboot.c?h=v6.19#n728)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/reboot.c?h=v6.18.18#n728)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n212)
pub fn reboot(magic: Int, magic_too: Int, cmd: Int, arg: *const Void) -> Int {
    // SAFETY: this wrapper forwards the raw user pointer without
    // dereferencing it in Rust. Linux 1.0 ignores the fourth argument
    // entirely, while current kernels validate or copy from it as part of the
    // syscall.
    unsafe {
        syscall4(
            Sysno::Reboot,
            magic as isize,
            magic_too as isize,
            cmd as isize,
            arg.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::Int;
    use core::ptr;

    use crate::arch::current::Sysno;

    use super::reboot;

    const LINUX_REBOOT_MAGIC1: u32 = 0xfee1dead;
    const LINUX_REBOOT_MAGIC2: Int = 672_274_793;
    const LINUX_REBOOT_CMD_RESTART2: Int = 0xA1B2C3D4_u32 as Int;

    #[test]
    fn test_reboot_syscall_number() {
        assert_eq!(Sysno::Reboot as isize, 88);
    }

    #[test]
    fn test_reboot_bad_magic_is_rejected_or_permission_denied() {
        let ret = reboot(0, 0, 0, ptr::null());
        let expected = [-1, -22];

        // Current kernels check privilege before validating the magic values,
        // so ordinary unprivileged runs stop at `EPERM`. Privileged test
        // environments can continue to the historical `EINVAL` check.
        assert!(
            expected.contains(&ret),
            "expected EPERM or EINVAL from reboot with bad magic, got {ret}",
        );
    }

    #[test]
    fn test_reboot_restart2_without_pointer_fails_or_needs_privilege() {
        let ret = reboot(
            LINUX_REBOOT_MAGIC1 as Int,
            LINUX_REBOOT_MAGIC2,
            LINUX_REBOOT_CMD_RESTART2,
            ptr::null(),
        );
        let expected = [-1, -14];

        assert!(
            expected.contains(&ret),
            "expected EPERM or EFAULT from reboot(RESTART2, null), got {ret}",
        );
    }
}
