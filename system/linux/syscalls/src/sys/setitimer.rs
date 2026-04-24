use celer_system_linux_ctypes::{Int, Itimerval};

use crate::arch::current::{Sysno, syscall3};

/// Set one of the process interval timers and optionally return its previous
/// state.
///
/// This wrapper spans the original Linux 1.0 x86 `setitimer` entry point and
/// the current native entrypoints exported by this crate on x86 and aarch64.
///
/// # Safety
/// - `value`, when non-null, must be valid to read one [`Itimerval`] value
///   for the duration of the syscall.
/// - `old_value`, when non-null, must be valid to write one [`Itimerval`]
///   value for the duration of the syscall.
/// - `old_value`, when non-null, must not alias live Rust references or other
///   memory that would violate Rust's aliasing rules while the kernel may
///   write through that pointer.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 accepts any `struct timeval` field values
///   that fit in the ABI struct and does not report copy-in failure from a
///   non-null `value`, while current kernels reject noncanonical timeval
///   fields with `EINVAL` and report bad non-null input or output pointers
///   with `EFAULT`.
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - `which` must be `ITIMER_REAL`, `ITIMER_VIRTUAL`, or `ITIMER_PROF`.
/// - `value == NULL` is accepted and behaves like a zeroed `Itimerval`,
///   disarming the selected timer.
/// - `old_value == NULL` is accepted and suppresses copy-out of the previous
///   timer state.
/// - On success, a non-null `old_value` receives the timer state from before
///   this call updated the selected timer.
/// - Linux 1.0 does not validate the `tv_sec` / `tv_usec` fields supplied in
///   a non-null `value`; current kernels reject negative seconds or
///   microseconds outside `0..1_000_000`.
///
/// # Errors
/// - `EINVAL`: `which` is not `ITIMER_REAL`, `ITIMER_VIRTUAL`, or
///   `ITIMER_PROF`.
/// - `EINVAL`: current kernels also return this when a non-null `value`
///   contains noncanonical timeval fields.
/// - `EFAULT`: Linux 1.0 reports this when copying a successful result to a
///   non-null `old_value` fails; current kernels may also return it for bad
///   non-null `value` or `old_value` pointers.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/setitimer.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/itimer.c?h=v7.0#n351)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n119)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n288)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/itimer.c?h=v6.18.18#n351)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n119)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n288)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/itimer.c?h=1.0#n110)
///
/// # Historical References
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n113)
pub unsafe fn setitimer(
    which: Int,
    value: *const Itimerval,
    old_value: *mut Itimerval,
) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe {
        syscall3(
            Sysno::Setitimer,
            which as isize,
            value.addr() as isize,
            old_value.addr() as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{
        ITIMER_PROF, ITIMER_REAL, ITIMER_VIRTUAL, Itimerval, Timeval,
    };

    use crate::sys::test_support::process_global_state_guard;

    use super::setitimer;
    use crate::arch::current::Sysno;

    fn zero_itimerval() -> Itimerval {
        Itimerval {
            it_interval: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            it_value: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        }
    }

    #[test]
    fn test_setitimer_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 104;
        #[cfg(target_arch = "aarch64")]
        let expected = 103;
        #[cfg(target_arch = "x86_64")]
        let expected = 38;

        assert_eq!(Sysno::Setitimer as isize, expected);
    }

    #[test]
    fn test_setitimer_layout() {
        #[cfg(target_arch = "x86")]
        let expected = (16, 4, 8);
        #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
        let expected = (32, 8, 16);

        assert_eq!(core::mem::size_of::<Itimerval>(), expected.0);
        assert_eq!(core::mem::align_of::<Itimerval>(), expected.1);
        assert_eq!(core::mem::offset_of!(Itimerval, it_interval), 0);
        assert_eq!(core::mem::offset_of!(Itimerval, it_value), expected.2);
    }

    #[test]
    fn test_setitimer_zero_value_succeeds() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        // SAFETY: `zero` is readable for one `Itimerval`; null `old_value`
        // is allowed.
        let rc = unsafe {
            setitimer(ITIMER_REAL, &raw const zero, core::ptr::null_mut())
        };

        assert_eq!(rc, 0, "setitimer(ITIMER_REAL, zero, NULL) failed: {rc}");
    }

    #[test]
    fn test_setitimer_none_value_clears_timer() {
        let _guard = process_global_state_guard();
        let mut old = Itimerval {
            it_interval: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
            it_value: Timeval {
                tv_sec: -1,
                tv_usec: -1,
            },
        };

        // SAFETY: null `value` is allowed; `old` is writable for one
        // `Itimerval`.
        let rc =
            unsafe { setitimer(ITIMER_REAL, core::ptr::null(), &raw mut old) };

        assert_eq!(
            rc, 0,
            "setitimer(ITIMER_REAL, NULL, &mut old) failed: {rc}"
        );
    }

    #[test]
    fn test_setitimer_second_clear_reports_zero_old_value() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();
        let mut old = zero_itimerval();

        // SAFETY: `zero` is readable for one `Itimerval`; null `old_value`
        // is allowed.
        let rc1 = unsafe {
            setitimer(ITIMER_REAL, &raw const zero, core::ptr::null_mut())
        };
        assert_eq!(rc1, 0, "initial clear failed: {rc1}");

        // SAFETY: `zero` is readable and `old` is writable for one
        // `Itimerval`.
        let rc2 =
            unsafe { setitimer(ITIMER_REAL, &raw const zero, &raw mut old) };

        assert_eq!(rc2, 0, "second clear failed: {rc2}");
        assert_eq!(old, zero);
    }

    #[test]
    fn test_setitimer_invalid_selector_returns_einval() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        // SAFETY: `zero` is readable for one `Itimerval`; null `old_value`
        // is allowed.
        let rc =
            unsafe { setitimer(3, &raw const zero, core::ptr::null_mut()) };

        assert_eq!(rc, -22, "setitimer(3, zero, NULL) returned {rc}");
    }

    #[test]
    fn test_setitimer_current_kernel_invalid_tv_usec_returns_einval() {
        let _guard = process_global_state_guard();
        let invalid = Itimerval {
            it_interval: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            it_value: Timeval {
                tv_sec: 0,
                tv_usec: 1_000_000,
            },
        };

        // SAFETY: `invalid` is readable for one `Itimerval`; null
        // `old_value` is allowed.
        let rc = unsafe {
            setitimer(ITIMER_REAL, &raw const invalid, core::ptr::null_mut())
        };

        assert_eq!(
            rc, -22,
            "setitimer(ITIMER_REAL, invalid, NULL) returned {rc}"
        );
    }

    #[test]
    fn test_setitimer_current_kernel_negative_tv_sec_returns_einval() {
        let _guard = process_global_state_guard();
        let invalid = Itimerval {
            it_interval: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            it_value: Timeval {
                tv_sec: -1,
                tv_usec: 0,
            },
        };

        // SAFETY: `invalid` is readable for one `Itimerval`; null
        // `old_value` is allowed.
        let rc = unsafe {
            setitimer(ITIMER_REAL, &raw const invalid, core::ptr::null_mut())
        };

        assert_eq!(
            rc, -22,
            "setitimer(ITIMER_REAL, invalid, NULL) returned {rc}"
        );
    }

    #[test]
    fn test_setitimer_supported_selectors_accept_zero_value() {
        let _guard = process_global_state_guard();
        let zero = zero_itimerval();

        for which in [ITIMER_REAL, ITIMER_VIRTUAL, ITIMER_PROF] {
            // SAFETY: `zero` is readable for one `Itimerval`; null
            // `old_value` is allowed.
            let rc = unsafe {
                setitimer(which, &raw const zero, core::ptr::null_mut())
            };

            assert_eq!(rc, 0, "setitimer({which}, zero, NULL) failed: {rc}");
        }
    }
}
