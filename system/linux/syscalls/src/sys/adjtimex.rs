use celer_system_linux_ctypes::{Int, Timex};

use crate::arch::current::{Sysno, syscall1};

/// Read or adjust kernel clock discipline parameters through `struct timex`.
///
/// This wrapper targets the native `sys_adjtimex` entrypoint. The same pointer
/// is used for both copy-in and copy-out, but Linux 1.0 copies only the
/// historical prefix through [`Timex::tick`], while current kernels copy the
/// full native layout.
///
/// # Safety
/// - `txc_p`, when non-null, must be valid for the kernel to read and later
///   write the copied ABI bytes for the duration of the syscall.
/// - Linux 1.0 touches the historical prefix through [`Timex::tick`].
/// - Current kernels touch the full native layout.
/// - The pointed-to memory must not violate Rust's aliasing rules while the
///   kernel may write through it.
///
/// # Kernel Support
/// - Introduced: Linux 1.0 on i386; present from the initial aarch64 syscall
///   table
/// - Behavior changes: current kernels add newer mode bits, extra read-only
///   fields, `TIME_WAIT` / `TIME_ERROR` return states, and a possible
///   `ENODEV` failure if the core clock is not valid
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 requires the superuser check whenever `modes != 0`.
/// - Current kernels require `CAP_SYS_TIME` for mutating modes, except the
///   internal read-only `ADJ_ADJTIME` case used by `ntp_adjtime`.
///
/// # Behavior
/// - `txc_p` points to an input/output [`Timex`] buffer.
/// - Linux 1.0 validates write access to and copies only the historical
///   prefix through [`Timex::tick`], because its `struct timex` ends there.
/// - Current kernels copy the full native layout, including the read-only tail
///   fields after [`Timex::tick`].
/// - With `modes == 0`, the syscall is a read-only query and returns the
///   current clock state code while filling the copied-out fields with current
///   values.
/// - With mutable mode bits set, Linux 1.0 may update offset, frequency,
///   error, status, time constant, and tick parameters before writing the
///   resulting state back out.
/// - Linux 1.0 returns one of `TIME_OK`, `TIME_INS`, `TIME_DEL`, `TIME_OOP`,
///   or `TIME_BAD` on success.
/// - Current kernels can still mutate kernel time state before a later
///   copy-out fault returns `EFAULT`.
///
/// # Errors
/// - `EFAULT`: `txc_p` is null or does not point to writable memory for the
///   bytes the kernel copies back out: through [`Timex::tick`] on Linux 1.0,
///   or the full native layout on current kernels.
/// - `EPERM`: Linux 1.0 rejects any nonzero `modes` from an unprivileged
///   caller.
/// - `EINVAL`: in Linux 1.0, `ADJ_OFFSET` supplies an out-of-range `offset`,
///   `ADJ_STATUS` supplies a status outside `TIME_OK..=TIME_BAD`, or
///   `ADJ_TICK` supplies a tick outside the accepted range.
/// - Current kernels may also return `EINVAL` for invalid `ADJ_ADJTIME`,
///   `ADJ_SETOFFSET`, or 64-bit-only frequency-overflow cases, and `ENODEV`
///   if the core clock is not valid.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/adjtimex.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/time/time.c?h=v7.0#n269)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/time/time.c?h=v6.18.18#n269)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/time.c?h=1.0#n288)
///
/// # Historical References
/// - Linux 1.0 `struct timex`:
///   [include/linux/timex.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/timex.h?h=1.0#n77)
pub unsafe fn adjtimex(txc_p: *mut Timex) -> Int {
    // SAFETY: guaranteed by caller.
    unsafe { syscall1(Sysno::Adjtimex, txc_p.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use celer_system_linux_ctypes::{
        ADJ_OFFSET, ADJ_TICK, TIME_DEL, TIME_ERROR, TIME_INS, TIME_OK,
        TIME_OOP, TIME_WAIT, Timeval, Timex,
    };

    use super::adjtimex;
    use crate::arch::current::Sysno;

    fn zero_timex() -> Timex {
        Timex {
            modes: 0,
            offset: 0,
            freq: 0,
            maxerror: 0,
            esterror: 0,
            status: 0,
            constant: 0,
            precision: 0,
            tolerance: 0,
            time: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            tick: 0,
            ppsfreq: 0,
            jitter: 0,
            shift: 0,
            stabil: 0,
            jitcnt: 0,
            calcnt: 0,
            errcnt: 0,
            stbcnt: 0,
            tai: 0,
            __padding: [0; 11],
        }
    }

    #[test]
    fn test_adjtimex_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 124;
        #[cfg(target_arch = "aarch64")]
        let expected = 171;

        assert_eq!(Sysno::Adjtimex as isize, expected);
    }

    #[test]
    fn test_adjtimex_layout() {
        #[cfg(target_arch = "x86")]
        let expected = (128, 4, 36, 44, 48, 80, 84);
        #[cfg(target_arch = "aarch64")]
        let expected = (208, 8, 72, 88, 96, 160, 164);

        assert_eq!(core::mem::size_of::<Timex>(), expected.0);
        assert_eq!(core::mem::align_of::<Timex>(), expected.1);
        assert_eq!(core::mem::offset_of!(Timex, modes), 0);
        assert_eq!(core::mem::offset_of!(Timex, time), expected.2);
        assert_eq!(core::mem::offset_of!(Timex, tick), expected.3);
        assert_eq!(core::mem::offset_of!(Timex, ppsfreq), expected.4);
        assert_eq!(core::mem::offset_of!(Timex, tai), expected.5);
        assert_eq!(core::mem::offset_of!(Timex, __padding), expected.6);
    }

    #[test]
    fn test_adjtimex_null_pointer_faults() {
        // SAFETY: this intentionally passes a null pointer to verify the
        // syscall reports `EFAULT`.
        let rc = unsafe { adjtimex(core::ptr::null_mut()) };

        assert_eq!(rc, -14, "expected EFAULT for null timex pointer, got {rc}");
    }

    #[test]
    fn test_adjtimex_query_succeeds_and_updates_buffer() {
        let mut txc = zero_timex();

        // SAFETY: `txc` is writable for the full `Timex` layout.
        let rc = unsafe { adjtimex(&raw mut txc) };

        assert!(
            matches!(
                rc,
                TIME_OK
                    | TIME_INS
                    | TIME_DEL
                    | TIME_OOP
                    | TIME_WAIT
                    | TIME_ERROR
            ),
            "unexpected adjtimex status {rc}"
        );
        assert!(txc.maxerror >= 0, "maxerror should be non-negative");
        assert!(txc.esterror >= 0, "esterror should be non-negative");
    }

    #[test]
    fn test_adjtimex_invalid_pointer_faults() {
        // SAFETY: this intentionally passes an invalid pointer to verify the
        // syscall reports `EFAULT`.
        let rc =
            unsafe { adjtimex(core::ptr::without_provenance_mut::<Timex>(1)) };

        assert_eq!(rc, -14, "expected EFAULT for bad timex pointer, got {rc}");
    }

    #[test]
    fn test_adjtimex_unprivileged_offset_change_is_rejected() {
        let mut txc = zero_timex();
        txc.modes = ADJ_OFFSET;
        txc.offset = 1;

        // SAFETY: `txc` is writable for the full `Timex` layout.
        let rc = unsafe { adjtimex(&raw mut txc) };

        assert_eq!(
            rc, -1,
            "expected EPERM for unprivileged ADJ_OFFSET, got {rc}"
        );
    }

    #[test]
    fn test_adjtimex_unprivileged_tick_change_is_rejected() {
        let mut txc = zero_timex();
        txc.modes = ADJ_TICK;
        txc.tick = 10_000;

        // SAFETY: `txc` is writable for the full `Timex` layout.
        let rc = unsafe { adjtimex(&raw mut txc) };

        assert_eq!(
            rc, -1,
            "expected EPERM for unprivileged ADJ_TICK, got {rc}"
        );
    }
}
