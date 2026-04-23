use celer_system_linux_ctypes::{Int, OldGidT};

use crate::arch::current::{Sysno, syscall2};

/// Replace the calling process's supplementary groups through the legacy i386
/// `setgroups16` ABI.
///
/// # Kernel Support
/// - Introduced: earliest verified available source is Linux 0.12
/// - Behavior changes: current i386 keeps syscall `81` as the legacy
///   `setgroups16` entry, which widens each 16-bit gid into the modern
///   credential subsystem before sorting and committing the new group list
/// - Availability: current x86 32-bit kernels expose syscall `81` as
///   `sys_setgroups16` when `CONFIG_UID16` is enabled; otherwise the slot can
///   fall back to `sys_ni_syscall`
///
/// # Required Privileges
/// - `CAP_SETGID` is required on current kernels, and the caller's user
///   namespace must permit `setgroups`.
/// - Linux 1.0 required superuser privileges through `suser()`.
///
/// # Behavior
/// - Replaces the caller's supplementary group list in one shot.
/// - `gidsetsize == 0` is valid and requests an empty supplementary-group set.
/// - Linux 1.0 copied up to `NGROUPS` 16-bit gids directly into
///   `current->groups[]` and terminated the list by setting the next slot to
///   `NOGROUP`.
/// - Current i386 routes syscall `81` through `sys_setgroups16`, validates
///   each gid in the caller's current user namespace, sorts the list, and
///   then installs it.
/// - Successful calls return `0`.
///
/// # Errors
/// - `EFAULT`: the kernel cannot read one of the gids from the supplied user
///   buffer.
/// - `EINVAL`: too many gids were supplied, or one of the gids is invalid in
///   the caller's current user namespace.
/// - `ENOMEM`: the kernel could not allocate the intermediate group or
///   credential structures.
/// - `ENOSYS`: builds without `CONFIG_UID16` may route syscall `81` to
///   `sys_ni_syscall` instead of `setgroups16`.
/// - `EPERM`: the caller lacks permission to replace its supplementary
///   groups, including user-namespace `setgroups` restrictions.
/// - LSM-specific negative errors may also be returned.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setgroups.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v7.0#n177)
/// - Stable table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n96)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n177)
/// - LTS table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n96)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n560)
///
/// # Historical References
/// - Earliest verified available source:
///   [Linux 0.12](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=0.12#n307)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn setgroups16(gidsetsize: Int, grouplist: *const OldGidT) -> Int {
    // SAFETY: this wrapper forwards the raw user pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe {
        syscall2(
            Sysno::Setgroups,
            gidsetsize as isize,
            grouplist.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{Int, OldGidT, PidT};

    use super::setgroups16;
    use crate::{
        arch::current::{Sysno, syscall2},
        sys::{exit, fork, getgid16, waitpid},
    };
    fn assert_child_matches_raw(groups: &[OldGidT]) {
        let pid = fork();
        assert!(pid >= 0, "fork failed: {pid}");

        fn handle_pid(pid: PidT, groups: &[OldGidT]) {
            if pid == 0 {
                let gidsetsize = groups.len() as Int;
                let grouplist = if groups.is_empty() {
                    ptr::null()
                } else {
                    groups.as_ptr()
                };
                let wrapped = setgroups16(gidsetsize, grouplist);
                let raw = unsafe {
                    syscall2(
                        Sysno::Setgroups,
                        gidsetsize as isize,
                        grouplist.addr() as isize,
                    ) as i32
                };
                exit(if wrapped == raw { 0 } else { 1 });
            }
        }

        handle_pid(pid, groups);

        let mut status = 0;
        let waited = unsafe { waitpid(pid, &raw mut status, 0) };
        assert_eq!(waited, pid);
        assert_eq!(status & 0x7f, 0);
        assert_eq!((status >> 8) & 0xff, 0);
    }

    #[test]
    fn test_setgroups16_empty_slice_matches_raw_syscall() {
        assert_child_matches_raw(&[]);
    }

    #[test]
    fn test_setgroups16_single_current_gid_matches_raw_syscall() {
        let groups = [getgid16()];

        assert_child_matches_raw(&groups);
    }
}
