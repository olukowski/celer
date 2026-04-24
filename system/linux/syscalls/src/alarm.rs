use celer_system_linux_ctypes::UnsignedInt;

use crate::sys;

/// Arm the process alarm timer and return the remaining seconds on any prior alarm.
///
/// On success, returns the whole seconds remaining on a previously armed alarm,
/// or `0` when no alarm was pending.
///
/// See [`sys::alarm`] for kernel behavior and source references.
pub fn alarm(seconds: UnsignedInt) -> UnsignedInt {
    let ret = sys::alarm(seconds);

    ret as UnsignedInt
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::sys::test_support::process_global_state_guard;

    use super::alarm;

    #[test]
    fn test_alarm_roundtrip() {
        let _guard = process_global_state_guard();

        let old = alarm(2);
        let cleared = alarm(0);

        assert!(old <= 2);
        assert!(cleared <= 2);
    }
}
