use celer_system_linux_ctypes::{Int, Void};

use crate::arch::linux_1_0::{Sysno, syscall1};

/// Run the historical Linux 1.0 bootstrap-only `setup` syscall.
///
/// This wrapper is exposed as [`crate::sys::linux_1_0::setup`], not as a
/// default top-level syscall export, because syscall number `0` changed
/// meaning after Linux 1.0.
///
/// Linux 1.0 still exposes syscall number `0` as `setup`, but current x86
/// Linux uses syscall number `0` for `restart_syscall` instead.
///
/// # Kernel Support
/// - Introduced: Linux 0.10
/// - Behavior changes: Linux 0.10 read BIOS drive data from `bios`; Linux 1.0
///   ignores the pointer and performs one-shot block-device setup instead.
/// - Availability: correct only for Linux 1.0 x86 kernels; current x86 Linux
///   syscall tables use syscall number `0` for `restart_syscall`
///
/// # Required Privileges
/// - None beyond reaching the historical one-shot init-only syscall entry.
///
/// # Behavior
/// - Linux 1.0 ignores `bios`.
/// - Linux 1.0 allows the syscall body to run only once after boot.
/// - The successful path walks the registered block-device list, optionally
///   loads a ramdisk, then mounts the root filesystem.
/// - Failures in the root-mount helper are not reported as errno returns; the
///   kernel panics instead.
///
/// # Errors
/// - Linux 1.0 returns literal `-1` when its one-shot guard rejects a
///   repeated call. This is errno-shaped like `EPERM`, but the source does not
///   spell it as `-EPERM`.
///
/// # References
/// - Linux 1.0 syscall number table:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n9)
/// - Linux 1.0 implementation:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/drivers/block/genhd.c?h=1.0#n197)
/// - Current x86-32 syscall table:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.19#n15)
/// - LTS x86-32 syscall table:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n15)
/// - Current `restart_syscall` implementation:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/signal.c?h=v6.19#n3173)
/// - LTS `restart_syscall` implementation:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/signal.c?h=v6.18.18#n3175)
///
/// # Historical References
/// - First appearance:
///   [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/unistd.h?h=0.10#n60)
/// - Linux 0.10 implementation:
///   [Linux 0.10](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/blk_drv/hd.c?h=0.10#n58)
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn setup(bios: *mut Void) -> Int {
    // SAFETY: Linux 1.0 does not dereference `bios`, and this wrapper only
    // forwards the raw historical ABI argument.
    unsafe { syscall1(Sysno::Setup, bios.addr() as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::arch::linux_1_0::Sysno;

    #[test]
    fn test_setup_syscall_number() {
        assert_eq!(Sysno::Setup as isize, 0);
    }
}
