use celer_system_linux_ctypes::Long;

use crate::arch::current::{Sysno, syscall0};

/// Suspend the calling thread until a signal is pending for delivery.
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
/// - The call blocks in interruptible sleep until a signal is pending.
/// - On x86 Linux, the kernel returns `EINTR` after the signal-return path
///   converts the syscall's internal restart code.
///
/// # Errors
/// - `EINTR`: a signal interrupted the sleep and the kernel did not restart
///   the syscall.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/pause.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n4820)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n4819)
/// - x86 signal-return conversion:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/signal.c?h=v6.19#n263)
///
/// # Historical References
/// - First appearance: [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sched.c?h=0.10#n138)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn pause() -> Long {
    // SAFETY: pause takes no arguments and has no caller-side preconditions.
    syscall0(Sysno::Pause) as Long
}
