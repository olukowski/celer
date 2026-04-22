use celer_system_linux_ctypes::{Int, OldGidT};

use crate::arch::current::{Sysno, syscall2};

/// Return the supplementary group IDs through the legacy i386 `getgroups16`
/// ABI.
///
/// Linux 1.0 stored supplementary groups as 16-bit `gid_t` values and
/// returned the number of valid entries even when the caller-provided buffer
/// was too small. Current x86 kernels keep syscall number 80 as the legacy
/// `getgroups16` entrypoint, but now reject negative or undersized
/// `gidsetsize` values with `EINVAL`.
///
/// # Safety
/// - On current x86 kernels, if `gidsetsize` is nonzero and at least the
///   current supplementary group count, `grouplist` must be valid to write
///   that many consecutive [`OldGidT`] values for the duration of the
///   syscall.
/// - On Linux 1.0, if `gidsetsize` is nonzero, `grouplist` had to be valid
///   for `gidsetsize` consecutive [`OldGidT`] values because the syscall
///   validated the full requested span before truncating any copy.
/// - `grouplist` may be null only when no write occurs.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 truncated to `gidsetsize` and returned the
///   full copied count, while current x86 kernels keep the 16-bit ABI as
///   `sys_getgroups16` and reject undersized or negative counts with
///   `EINVAL`
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `gidsetsize` counts [`OldGidT`] elements, not bytes.
/// - When `gidsetsize == 0`, the kernel returns the current supplementary
///   group count without writing to `grouplist`.
/// - On Linux 1.0, if `gidsetsize` was smaller than the current group count,
///   the kernel copied only `gidsetsize` entries and returned that truncated
///   count.
/// - On current x86 kernels, if `gidsetsize` is nonzero and smaller than the
///   current group count, the syscall returns `EINVAL` without writing a
///   partial list.
///
/// # Errors
/// - `EINVAL`: `gidsetsize` is negative on current x86 kernels.
/// - `EINVAL`: `gidsetsize` is nonzero but smaller than the current
///   supplementary group count on current x86 kernels.
/// - `EFAULT`: on current x86 kernels, `gidsetsize` is nonzero, large enough
///   for the current supplementary group count, and copying that many
///   [`OldGidT`] values to `grouplist` fails.
/// - `EFAULT`: on Linux 1.0, `gidsetsize` is nonzero and `grouplist` is not
///   writable for `gidsetsize` [`OldGidT`] values during the initial
///   `verify_area` check.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/getgroups.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/uid16.c?h=v6.19#n154)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/uid16.c?h=v6.18.18#n154)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n540)
///
/// # Historical References
/// - Linux 1.0 syscall number table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n89)
/// - Linux 1.0 `gid_t` definition:
///   [include/linux/types.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/types.h?h=1.0#n35)
pub unsafe fn getgroups16(gidsetsize: Int, grouplist: *mut OldGidT) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall2(
            Sysno::Getgroups,
            gidsetsize as isize,
            grouplist.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
mod tests {
    use celer_system_linux_ctypes::{Int, OldGidT};

    use crate::arch::current::{Sysno, syscall2};

    use super::getgroups16;

    #[test]
    fn test_getgroups_sysno() {
        assert_eq!(Sysno::Getgroups as isize, 80);
    }

    #[test]
    fn test_getgroups_probe_with_null_pointer() {
        // SAFETY: `grouplist` may be null when `gidsetsize == 0`.
        let count = unsafe { getgroups16(0, core::ptr::null_mut()) };

        assert!(
            count >= 0,
            "getgroups16 count probe should succeed, got {count}"
        );
    }

    #[test]
    fn test_getgroups_exact_sized_buffer_succeeds() {
        // SAFETY: `grouplist` may be null when `gidsetsize == 0`.
        let count = unsafe { getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "getgroups16 count probe failed: {count}");

        let mut groups = vec![OldGidT::MAX; count as usize];
        let ptr = if groups.is_empty() {
            core::ptr::null_mut()
        } else {
            groups.as_mut_ptr()
        };

        // SAFETY: `ptr` is null only for the zero-sized probe case;
        // otherwise it is writable for `count` entries.
        let ret = unsafe { getgroups16(count, ptr) };

        assert_eq!(ret, count, "getgroups16 exact-sized call failed: {ret}");
    }

    #[test]
    fn test_getgroups_oversized_buffer_preserves_tail() {
        // SAFETY: `grouplist` may be null when `gidsetsize == 0`.
        let count = unsafe { getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "getgroups16 count probe failed: {count}");

        let mut groups = vec![OldGidT::MAX; count as usize + 1];

        // SAFETY: `groups` is writable for `count + 1` entries.
        let ret = unsafe { getgroups16(count + 1, groups.as_mut_ptr()) };

        assert_eq!(ret, count, "getgroups16 oversized call failed: {ret}");
        assert_eq!(
            groups[count as usize],
            OldGidT::MAX,
            "kernel should not write beyond the returned group count"
        );
    }

    #[test]
    fn test_getgroups_negative_size_returns_einval() {
        // SAFETY: `grouplist` may be null when the kernel rejects the count
        // before attempting any write.
        let ret = unsafe { getgroups16(-1, core::ptr::null_mut()) };

        assert_eq!(
            ret, -22,
            "expected EINVAL for negative gidsetsize, got {ret}"
        );
    }

    #[test]
    fn test_getgroups_undersized_buffer_returns_einval_when_nonempty() {
        // SAFETY: `grouplist` may be null when `gidsetsize == 0`.
        let count = unsafe { getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "getgroups16 count probe failed: {count}");

        if count == 0 {
            return;
        }

        let mut groups = vec![OldGidT::MAX; count as usize];

        // SAFETY: `groups` is writable for one fewer entry than requested by
        // the kernel, which should trigger `EINVAL` before any write.
        let ret = unsafe { getgroups16(count - 1, groups.as_mut_ptr()) };

        assert_eq!(
            ret, -22,
            "expected EINVAL for undersized gidsetsize, got {ret}"
        );
        assert!(
            groups.iter().all(|&group| group == OldGidT::MAX),
            "EINVAL should leave the caller buffer unchanged"
        );
    }

    #[test]
    fn test_getgroups_invalid_pointer_returns_efault() {
        // SAFETY: `grouplist` may be null when `gidsetsize == 0`.
        let count = unsafe { getgroups16(0, core::ptr::null_mut()) };
        assert!(count >= 0, "getgroups16 count probe failed: {count}");

        if count == 0 {
            return;
        }

        // SAFETY: this intentionally passes an invalid writable pointer to
        // verify the kernel's `EFAULT` path.
        let ret = unsafe { getgroups16(count, usize::MAX as *mut OldGidT) };

        assert_eq!(
            ret, -14,
            "expected EFAULT for invalid group buffer, got {ret}"
        );
    }

    #[test]
    fn test_getgroups_negative_size_matches_raw_syscall() {
        // SAFETY: `grouplist` may be null when the kernel rejects the count
        // before attempting any write.
        let wrapped = unsafe { getgroups16(-1, core::ptr::null_mut()) };
        // SAFETY: same invalid argument combination as above.
        let raw = unsafe { syscall2(Sysno::Getgroups, -1, 0) as Int };

        assert_eq!(wrapped, -22, "wrapped getgroups16 should return EINVAL");
        assert_eq!(raw, -22, "raw getgroups syscall should return EINVAL");
    }
}
